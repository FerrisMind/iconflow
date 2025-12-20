#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


MANIFEST_PATH = Path("ASSETS_MANIFEST.json")
ASSETS_ROOT = Path("assets")


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(8192), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_manifest() -> dict[str, str]:
    if not MANIFEST_PATH.exists():
        raise FileNotFoundError(f"Manifest not found: {MANIFEST_PATH}")
    data = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError("Manifest must be a JSON object mapping paths to sha256 strings")
    normalized: dict[str, str] = {}
    for key, value in data.items():
        if not isinstance(key, str) or not isinstance(value, str):
            raise ValueError("Manifest keys and values must be strings")
        normalized[key] = value
    return normalized


def collect_assets(repo_root: Path) -> dict[str, Path]:
    if not ASSETS_ROOT.exists():
        raise FileNotFoundError(f"Assets directory not found: {ASSETS_ROOT}")
    assets: dict[str, Path] = {}
    for path in sorted(ASSETS_ROOT.rglob("*.ttf")):
        resolved = path.resolve()
        rel = resolved.relative_to(repo_root).as_posix()
        assets[rel] = resolved
    return assets


def main() -> int:
    repo_root = Path.cwd().resolve()
    manifest = load_manifest()
    assets = collect_assets(repo_root)

    asset_paths = set(assets.keys())
    manifest_paths = set(manifest.keys())

    missing = sorted(asset_paths - manifest_paths)
    if missing:
        print("Missing manifest entries for assets:", file=sys.stderr)
        for path in missing:
            print(f"  {path}", file=sys.stderr)
        return 1

    extra = sorted(manifest_paths - asset_paths)
    if extra:
        print("Manifest entries without assets:", file=sys.stderr)
        for path in extra:
            print(f"  {path}", file=sys.stderr)
        return 1

    mismatched = []
    for rel_path, asset_path in assets.items():
        actual = sha256_file(asset_path)
        expected = manifest[rel_path]
        if actual != expected:
            mismatched.append((rel_path, expected, actual))

    if mismatched:
        print("Asset hash mismatches detected:", file=sys.stderr)
        for rel_path, expected, actual in mismatched:
            print(f"  {rel_path}", file=sys.stderr)
            print(f"    expected: {expected}", file=sys.stderr)
            print(f"    actual:   {actual}", file=sys.stderr)
        return 1

    print(f"Integrity verified for {len(assets)} assets.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
