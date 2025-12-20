#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

from fontTools.ttLib import TTFont

try:
    import jsonschema
except ImportError as exc:  # pragma: no cover
    raise SystemExit("Missing dependency: jsonschema. Install with `python -m pip install jsonschema`.") from exc


SCHEMA_PATH = Path("assets/schema/iconflow-pack.schema.json")
MAPS_DIR = Path("assets/maps")
SURROGATE_MIN = 0xD800
SURROGATE_MAX = 0xDFFF
CARGO_TOML = Path("Cargo.toml")


def load_schema() -> dict:
    if not SCHEMA_PATH.exists():
        raise FileNotFoundError(f"Schema not found: {SCHEMA_PATH}")
    return json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))


def load_map(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def assert_no_surrogates(codepoint: int, path: Path, context: str) -> None:
    if SURROGATE_MIN <= codepoint <= SURROGATE_MAX:
        raise ValueError(f"{path}: surrogate codepoint in {context}: {hex(codepoint)}")


def get_ttf_family(path: Path) -> set[str]:
    font = TTFont(path)
    families = set()
    for record in font["name"].names:
        if record.nameID == 1:
            try:
                families.add(record.toUnicode())
            except Exception as exc:  # pragma: no cover
                raise ValueError(f"Unable to decode family name in {path}: {exc}") from exc
    return families


def load_features() -> set[str]:
    if not CARGO_TOML.exists():
        raise FileNotFoundError(f"Cargo.toml not found: {CARGO_TOML}")
    try:
        import tomllib
        data = tomllib.loads(CARGO_TOML.read_text(encoding="utf-8"))
    except ImportError:
        try:
            import tomli
        except ImportError as exc:  # pragma: no cover
            raise SystemExit(
                "Missing tomllib/tomli. Install tomli or use Python 3.11+."
            ) from exc
        data = tomli.loads(CARGO_TOML.read_text(encoding="utf-8"))
    features = data.get("features", {})
    if not isinstance(features, dict):
        raise ValueError("Invalid Cargo.toml: [features] must be a table")
    return set(features.keys())


def validate_map(path: Path, schema: dict, features: set[str]) -> None:
    data = load_map(path)
    jsonschema.Draft202012Validator(schema).validate(data)

    variant_ids = [v["id"] for v in data["variants"]]
    if len(variant_ids) != len(set(variant_ids)):
        raise ValueError(f"{path}: duplicate variant.id values")

    icon_names = [i["name"] for i in data["icons"]]
    if len(icon_names) != len(set(icon_names)):
        raise ValueError(f"{path}: duplicate icon.name values")

    variant_id_set = set(variant_ids)

    for variant in data["variants"]:
        feature = variant.get("feature")
        if feature is not None:
            if not isinstance(feature, str) or not feature.strip():
                raise ValueError(f"{path}: variant.feature must be a non-empty string")
            if feature not in features:
                raise ValueError(f"{path}: variant.feature '{feature}' not found in Cargo.toml")

        ttf_path = Path(variant["ttf_asset_path"])
        if not ttf_path.exists():
            raise FileNotFoundError(f"{path}: missing TTF at {ttf_path}")

        families = get_ttf_family(ttf_path)
        if variant["family"] not in families:
            raise ValueError(
                f"{path}: family '{variant['family']}' not found in {ttf_path} (found: {sorted(families)})"
            )

    for icon in data["icons"]:
        name = icon["name"]
        if "codepoint" in icon:
            assert_no_surrogates(icon["codepoint"], path, f"icon '{name}'")

        overrides = icon.get("overrides", {})
        if overrides:
            unknown = set(overrides.keys()) - variant_id_set
            if unknown:
                raise ValueError(f"{path}: icon '{name}' overrides unknown variants: {sorted(unknown)}")

            for variant_id, cp in overrides.items():
                assert_no_surrogates(cp, path, f"icon '{name}' override '{variant_id}'")

        availability = icon.get("availability")
        if availability is not None:
            unknown = set(availability) - variant_id_set
            if unknown:
                raise ValueError(f"{path}: icon '{name}' availability unknown variants: {sorted(unknown)}")

            if overrides:
                missing = set(overrides.keys()) - set(availability)
                if missing:
                    raise ValueError(
                        f"{path}: icon '{name}' overrides not listed in availability: {sorted(missing)}"
                    )


def main(argv: list[str] | None = None) -> int:
    schema = load_schema()
    features = load_features()

    if not MAPS_DIR.exists():
        raise FileNotFoundError(f"Maps directory not found: {MAPS_DIR}")

    map_files = sorted(MAPS_DIR.glob("*.json"))
    if not map_files:
        raise FileNotFoundError(f"No map files found in {MAPS_DIR}")

    for path in map_files:
        validate_map(path, schema, features)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
