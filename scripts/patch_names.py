#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path

from fontTools.ttLib import TTFont


@dataclass(frozen=True)
class PatchTarget:
    path: Path
    family: str


ASSETS_FONTS_DIR = Path("assets/fonts")

PACK_TITLES = {
    "bootstrap": "Bootstrap",
    "carbon": "Carbon",
    "devicon": "Devicon",
    "feather": "Feather",
    "fluentui": "Fluent UI",
    "heroicons": "Heroicons",
    "iconoir": "Iconoir",
    "ionicons": "Ionicons",
    "lobe": "Lobe",
    "lucide": "Lucide",
    "octicons": "Octicons",
    "phosphor": "Phosphor",
    "remixicon": "Remix Icon",
    "tabler": "Tabler",
}

STYLE_TOKENS = {
    "regular": "Regular",
    "filled": "Filled",
    "outline": "Outline",
    "light": "Light",
    "thin": "Thin",
    "bold": "Bold",
    "duotone": "Duotone",
    "glyph": "Glyph",
    "sharp": "Sharp",
}


def build_postscript_name(family: str) -> str:
    compact = family.replace(" ", "")
    sanitized = re.sub(r"[^A-Za-z0-9-]", "", compact)
    if not sanitized:
        raise ValueError(f"PostScript name is empty after sanitizing '{family}'")
    return sanitized


def set_name(name_table, name_id: int, value: str) -> None:
    existing = [n for n in name_table.names if n.nameID == name_id]
    if existing:
        for rec in existing:
            name_table.setName(value, name_id, rec.platformID, rec.platEncID, rec.langID)
        return
    # Fallback: add common Windows + Macintosh records.
    name_table.setName(value, name_id, 3, 1, 0x409)
    name_table.setName(value, name_id, 1, 0, 0)


def patch_font(path: Path, family: str) -> None:
    if not path.exists():
        raise FileNotFoundError(f"TTF not found: {path}")

    font = TTFont(path)
    name_table = font["name"]

    postscript = build_postscript_name(family)
    subfamily = "Regular"

    set_name(name_table, 1, family)   # Font Family
    set_name(name_table, 2, subfamily)  # Subfamily
    set_name(name_table, 4, family)   # Full name
    set_name(name_table, 6, postscript)  # PostScript name
    set_name(name_table, 16, family)  # Preferred Family
    set_name(name_table, 17, subfamily)  # Preferred Subfamily

    font.save(path)


def parse_targets_file(path: Path) -> list[PatchTarget]:
    import json

    data = json.loads(path.read_text(encoding="utf-8"))
    targets: list[PatchTarget] = []
    for item in data:
        if "path" not in item or "family" not in item:
            raise ValueError("Each target must include 'path' and 'family'")
        targets.append(PatchTarget(Path(item["path"]), item["family"]))
    return targets


def infer_family(pack_id: str, stem: str) -> str:
    if pack_id not in PACK_TITLES:
        raise ValueError(f"Unknown pack '{pack_id}' for {stem}")
    pack_title = PACK_TITLES[pack_id]
    suffix = stem
    if stem.startswith(f"{pack_id}-"):
        suffix = stem[len(pack_id) + 1 :]

    if pack_id == "heroicons":
        if suffix == "mini":
            return f"{pack_title} Filled Mini"
        if suffix == "tiny":
            return f"{pack_title} Filled Tiny"
        if suffix == "outline":
            return f"{pack_title} Outline"
        if suffix == "filled":
            return f"{pack_title} Filled"
    if pack_id == "octicons":
        if suffix == "tiny":
            return f"{pack_title} Regular Tiny"
        if suffix == "regular":
            return f"{pack_title} Regular"
    if pack_id == "fluentui":
        if suffix == "resizable":
            return f"{pack_title} Regular Resizable"
        style = STYLE_TOKENS.get(suffix)
        if style:
            return f"{pack_title} {style}"
    if pack_id == "remixicon":
        return f"{pack_title} Regular"

    style = STYLE_TOKENS.get(suffix, None)
    if style is None:
        style = "Regular"
    return f"{pack_title} {style}"


def collect_default_targets() -> list[PatchTarget]:
    targets: list[PatchTarget] = []
    if not ASSETS_FONTS_DIR.exists():
        raise FileNotFoundError(f"Assets fonts directory not found: {ASSETS_FONTS_DIR}")
    for pack_dir in sorted(ASSETS_FONTS_DIR.iterdir()):
        if not pack_dir.is_dir():
            continue
        pack_id = pack_dir.name
        for ttf in sorted(pack_dir.glob("*.ttf")):
            family = infer_family(pack_id, ttf.stem)
            targets.append(PatchTarget(ttf, family))
    return targets


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Patch TTF name table with unique family names.")
    parser.add_argument("--file", type=Path, help="Path to a TTF file to patch")
    parser.add_argument("--family", type=str, help="New family name for --file")
    parser.add_argument("--targets", type=Path, help="JSON array of {path,family} entries")
    parser.add_argument(
        "--apply-defaults",
        action="store_true",
        help="Patch built-in heroicons/bootstrap targets",
    )
    args = parser.parse_args(argv)

    targets: list[PatchTarget] = []
    if args.apply_defaults:
        targets.extend(collect_default_targets())
    if args.targets:
        targets.extend(parse_targets_file(args.targets))
    if args.file or args.family:
        if not (args.file and args.family):
            raise ValueError("--file and --family must be used together")
        targets.append(PatchTarget(args.file, args.family))

    if not targets:
        raise ValueError("No targets provided. Use --apply-defaults or --file/--family.")

    for target in targets:
        patch_font(target.path, target.family)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
