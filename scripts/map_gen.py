#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Dict, Iterable, List, Tuple

from fontTools.ttLib import TTFont


ASSETS_FONTS_DIR = Path("assets/fonts")
MAPS_DIR = Path("assets/maps")

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

NAME_PATTERN = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")


def load_family(path: Path) -> str:
    font = TTFont(path)
    families = []
    for record in font["name"].names:
        if record.nameID == 1:
            try:
                families.append(record.toUnicode())
            except Exception as exc:
                raise ValueError(f"Unable to decode family name in {path}: {exc}") from exc
    families = sorted(set(families))
    if not families:
        raise ValueError(f"No family names found in {path}")
    if len(families) > 1:
        raise ValueError(f"Multiple family names found in {path}: {families}")
    return families[0]


def load_cmap(path: Path) -> Dict[str, int]:
    font = TTFont(path)
    cmap = font.getBestCmap() or {}
    name_to_cp: Dict[str, int] = {}
    for cp, name in cmap.items():
        if name in name_to_cp:
            if name_to_cp[name] == cp:
                continue
            existing = name_to_cp[name]
            chosen = min(existing, cp)
            name_to_cp[name] = chosen
            print(
                f"Warning: duplicate glyph name '{name}' in {path} ({existing} vs {cp}); keeping {chosen}",
                file=sys.stderr,
            )
            continue
        name_to_cp[name] = cp
    return name_to_cp


def normalize_kebab(name: str) -> str:
    cleaned = name.strip().lower().replace("_", "-")
    cleaned = re.sub(r"[^a-z0-9-]+", "-", cleaned)
    cleaned = re.sub(r"-{2,}", "-", cleaned)
    cleaned = cleaned.strip("-")
    if not cleaned or not NAME_PATTERN.match(cleaned):
        raise ValueError(f"Invalid icon name after normalization: '{name}' -> '{cleaned}'")
    return cleaned


def merge_variants(
    variants: List[dict],
    variant_maps: Dict[str, Dict[str, int]],
    default_variant_id: str | None,
) -> List[dict]:
    variant_ids = [variant["id"] for variant in variants]
    all_names = sorted({name for names in variant_maps.values() for name in names})
    icons: List[dict] = []

    for name in all_names:
        available = [vid for vid in variant_ids if name in variant_maps.get(vid, {})]
        if not available:
            continue
        default_cp = None
        if default_variant_id and name in variant_maps.get(default_variant_id, {}):
            default_cp = variant_maps[default_variant_id][name]

        overrides: Dict[str, int] = {}
        if default_cp is None:
            for vid in available:
                overrides[vid] = variant_maps[vid][name]
        else:
            for vid in available:
                if vid == default_variant_id:
                    continue
                cp = variant_maps[vid][name]
                if cp != default_cp:
                    overrides[vid] = cp

        icon_entry = {"name": name}
        if default_cp is not None:
            icon_entry["codepoint"] = default_cp
        if overrides:
            icon_entry["overrides"] = overrides
        if len(available) != len(variant_ids):
            icon_entry["availability"] = available
        icons.append(icon_entry)

    return icons


def write_map(pack_id: str, variants: List[dict], icons: List[dict]) -> None:
    MAPS_DIR.mkdir(parents=True, exist_ok=True)
    payload = {
        "pack_id": pack_id,
        "variants": sorted(variants, key=lambda v: v["id"]),
        "icons": sorted(icons, key=lambda i: i["name"]),
    }
    path = MAPS_DIR / f"{pack_id}.json"
    path.write_text(json.dumps(payload, ensure_ascii=True, indent=2) + "\n", encoding="utf-8")


def ttf_path(pack_id: str, filename: str) -> Path:
    return ASSETS_FONTS_DIR / pack_id / filename


def load_icomoon_selection(path: Path) -> Dict[str, int]:
    data = json.loads(path.read_text(encoding="utf-8"))
    icons = data.get("icons", [])
    name_to_cp: Dict[str, int] = {}
    for icon in icons:
        props = icon.get("properties", {})
        name = props.get("name")
        code = props.get("code")
        if not isinstance(name, str) or not isinstance(code, int):
            raise ValueError(f"Invalid icon entry in {path}")
        normalized = normalize_kebab(name)
        if normalized in name_to_cp and name_to_cp[normalized] != code:
            raise ValueError(f"Duplicate icon name '{normalized}' in {path}")
        name_to_cp[normalized] = code
    return name_to_cp


def parse_remixicon_glyphs(path: Path) -> Tuple[Dict[str, int], Dict[str, int]]:
    data = json.loads(path.read_text(encoding="utf-8"))
    outline: Dict[str, int] = {}
    filled: Dict[str, int] = {}
    for raw_name, payload in data.items():
        if raw_name.endswith("-line"):
            base = normalize_kebab(raw_name[:-5])
            target = outline
        elif raw_name.endswith("-fill"):
            base = normalize_kebab(raw_name[:-5])
            target = filled
        else:
            continue
        unicode = payload.get("unicode", "")
        if not unicode.startswith("&#x") or not unicode.endswith(";"):
            raise ValueError(f"Unexpected unicode format for {raw_name} in {path}")
        codepoint = int(unicode[3:-1], 16)
        if base in target and target[base] != codepoint:
            raise ValueError(f"Duplicate icon name '{base}' in {path}")
        target[base] = codepoint
    return outline, filled


def parse_fluent_name(name: str) -> Tuple[str, int, str] | None:
    match = re.match(r"^ic_fluent_(.+)_(\d+)_(regular|filled|light)$", name)
    if not match:
        return None
    raw_name, raw_size, raw_style = match.groups()
    icon_name = normalize_kebab(raw_name.replace("_", "-"))
    return icon_name, int(raw_size), raw_style


def generate_bootstrap() -> Tuple[List[dict], List[dict]]:
    regular_path = ttf_path("bootstrap", "bootstrap-regular.ttf")
    filled_path = ttf_path("bootstrap", "bootstrap-filled.ttf")
    regular_map = {normalize_kebab(name): cp for name, cp in load_cmap(regular_path).items()}
    filled_map = {
        normalize_kebab(name[:-5] if name.endswith("-fill") else name): cp
        for name, cp in load_cmap(filled_path).items()
    }
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(regular_path),
            "ttf_asset_path": str(regular_path).replace("\\", "/"),
        },
        {
            "id": "filled",
            "style": "Filled",
            "size": "Regular",
            "family": load_family(filled_path),
            "ttf_asset_path": str(filled_path).replace("\\", "/"),
        },
    ]
    icons = merge_variants(variants, {"regular": regular_map, "filled": filled_map}, "regular")
    return variants, icons


def generate_heroicons() -> Tuple[List[dict], List[dict]]:
    outline_path = ttf_path("heroicons", "heroicons-outline.ttf")
    filled_path = ttf_path("heroicons", "heroicons-filled.ttf")
    mini_path = ttf_path("heroicons", "heroicons-mini.ttf")
    tiny_path = ttf_path("heroicons", "heroicons-tiny.ttf")
    outline_map = {normalize_kebab(name): cp for name, cp in load_cmap(outline_path).items()}
    filled_map = {normalize_kebab(name): cp for name, cp in load_cmap(filled_path).items()}
    mini_map = {normalize_kebab(name): cp for name, cp in load_cmap(mini_path).items()}
    tiny_map = {normalize_kebab(name): cp for name, cp in load_cmap(tiny_path).items()}
    variants = [
        {
            "id": "outline",
            "style": "Outline",
            "size": "Regular",
            "family": load_family(outline_path),
            "ttf_asset_path": str(outline_path).replace("\\", "/"),
        },
        {
            "id": "filled",
            "style": "Filled",
            "size": "Regular",
            "family": load_family(filled_path),
            "ttf_asset_path": str(filled_path).replace("\\", "/"),
        },
        {
            "id": "mini",
            "style": "Filled",
            "size": "Mini",
            "family": load_family(mini_path),
            "ttf_asset_path": str(mini_path).replace("\\", "/"),
            "feature": "heroicons-mini",
        },
        {
            "id": "tiny",
            "style": "Filled",
            "size": "Tiny",
            "family": load_family(tiny_path),
            "ttf_asset_path": str(tiny_path).replace("\\", "/"),
            "feature": "heroicons-tiny",
        },
    ]
    icons = merge_variants(
        variants,
        {"outline": outline_map, "filled": filled_map, "mini": mini_map, "tiny": tiny_map},
        "outline",
    )
    return variants, icons


def generate_carbon() -> Tuple[List[dict], List[dict]]:
    def normalize_carbon(name: str) -> str:
        if name.endswith("--glyph"):
            name = name[: -len("--glyph")]
        name = name.replace("--", "-")
        return normalize_kebab(name)

    regular_path = ttf_path("carbon", "carbon-regular.ttf")
    filled_path = ttf_path("carbon", "carbon-filled.ttf")
    outline_path = ttf_path("carbon", "carbon-outline.ttf")
    glyph_path = ttf_path("carbon", "carbon-glyph.ttf")

    regular_map = {normalize_carbon(name): cp for name, cp in load_cmap(regular_path).items()}
    filled_map = {normalize_carbon(name): cp for name, cp in load_cmap(filled_path).items()}
    outline_map = {normalize_carbon(name): cp for name, cp in load_cmap(outline_path).items()}
    glyph_map = {normalize_carbon(name): cp for name, cp in load_cmap(glyph_path).items()}

    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(regular_path),
            "ttf_asset_path": str(regular_path).replace("\\", "/"),
        },
        {
            "id": "filled",
            "style": "Filled",
            "size": "Regular",
            "family": load_family(filled_path),
            "ttf_asset_path": str(filled_path).replace("\\", "/"),
        },
        {
            "id": "outline",
            "style": "Outline",
            "size": "Regular",
            "family": load_family(outline_path),
            "ttf_asset_path": str(outline_path).replace("\\", "/"),
        },
        {
            "id": "glyph",
            "style": "Glyph",
            "size": "Regular",
            "family": load_family(glyph_path),
            "ttf_asset_path": str(glyph_path).replace("\\", "/"),
        },
    ]
    icons = merge_variants(
        variants,
        {
            "regular": regular_map,
            "filled": filled_map,
            "outline": outline_map,
            "glyph": glyph_map,
        },
        "regular",
    )
    return variants, icons


def generate_devicon() -> Tuple[List[dict], List[dict]]:
    ttf = ttf_path("devicon", "devicon-regular.ttf")
    icon_map = load_icomoon_selection(Path("tp/devicon/icomoon.json"))
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(ttf),
            "ttf_asset_path": str(ttf).replace("\\", "/"),
        }
    ]
    icons = merge_variants(variants, {"regular": icon_map}, "regular")
    return variants, icons


def generate_feather() -> Tuple[List[dict], List[dict]]:
    ttf = ttf_path("feather", "feather-regular.ttf")
    icon_map = {normalize_kebab(name): cp for name, cp in load_cmap(ttf).items()}
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(ttf),
            "ttf_asset_path": str(ttf).replace("\\", "/"),
        }
    ]
    icons = merge_variants(variants, {"regular": icon_map}, "regular")
    return variants, icons


def generate_fluentui() -> Tuple[List[dict], List[dict]]:
    regular_path = ttf_path("fluentui", "fluentui-regular.ttf")
    filled_path = ttf_path("fluentui", "fluentui-filled.ttf")
    light_path = ttf_path("fluentui", "fluentui-light.ttf")
    resizable_path = ttf_path("fluentui", "fluentui-resizable.ttf")

    def split_fluent(path: Path) -> Tuple[Dict[Tuple[str, int], int], Dict[Tuple[str, int], int], Dict[Tuple[str, int], int]]:
        regular: Dict[Tuple[str, int], int] = {}
        filled: Dict[Tuple[str, int], int] = {}
        light: Dict[Tuple[str, int], int] = {}
        for name, cp in load_cmap(path).items():
            parsed = parse_fluent_name(name)
            if not parsed:
                continue
            icon_name, size, style = parsed
            key = (icon_name, size)
            if style == "regular":
                regular[key] = cp
            elif style == "filled":
                filled[key] = cp
            elif style == "light":
                light[key] = cp
        return regular, filled, light

    regular_map, _, _ = split_fluent(regular_path)
    _, filled_map, _ = split_fluent(filled_path)
    _, _, light_map = split_fluent(light_path)
    resizable_regular, resizable_filled, _ = split_fluent(resizable_path)

    variants: List[dict] = []
    variant_maps: Dict[str, Dict[str, int]] = {}

    def add_variant(style: str, size: int | str, path: Path, icon_map: Dict[Tuple[str, int], int]) -> None:
        if isinstance(size, int):
            size_value: int | str = size
            size_suffix = str(size)
        else:
            size_value = size
            size_suffix = size.lower()
        variant_id = f"{style.lower()}-{size_suffix}"
        variants.append(
            {
                "id": variant_id,
                "style": style,
                "size": size_value,
                "family": load_family(path),
                "ttf_asset_path": str(path).replace("\\", "/"),
            }
        )
        variant_maps[variant_id] = {
            name: cp for (name, icon_size), cp in icon_map.items() if icon_size == size
        }

    for size in sorted({size for _, size in regular_map.keys()}):
        add_variant("Regular", size, regular_path, regular_map)
    for size in sorted({size for _, size in filled_map.keys()}):
        add_variant("Filled", size, filled_path, filled_map)
    for size in sorted({size for _, size in light_map.keys()}):
        add_variant("Light", size, light_path, light_map)

    resizable_regular_map = {name: cp for (name, _), cp in resizable_regular.items()}
    resizable_filled_map = {name: cp for (name, _), cp in resizable_filled.items()}
    variants.append(
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(resizable_path),
            "ttf_asset_path": str(resizable_path).replace("\\", "/"),
        }
    )
    variant_maps["regular"] = resizable_regular_map
    variants.append(
        {
            "id": "filled",
            "style": "Filled",
            "size": "Regular",
            "family": load_family(resizable_path),
            "ttf_asset_path": str(resizable_path).replace("\\", "/"),
        }
    )
    variant_maps["filled"] = resizable_filled_map

    icons = merge_variants(variants, variant_maps, "regular")
    return variants, icons


def generate_iconoir() -> Tuple[List[dict], List[dict]]:
    regular_path = ttf_path("iconoir", "iconoir-regular.ttf")
    filled_path = ttf_path("iconoir", "iconoir-filled.ttf")
    regular_map = {normalize_kebab(name): cp for name, cp in load_cmap(regular_path).items()}
    filled_map = {normalize_kebab(name): cp for name, cp in load_cmap(filled_path).items()}
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(regular_path),
            "ttf_asset_path": str(regular_path).replace("\\", "/"),
        },
        {
            "id": "filled",
            "style": "Filled",
            "size": "Regular",
            "family": load_family(filled_path),
            "ttf_asset_path": str(filled_path).replace("\\", "/"),
        },
    ]
    icons = merge_variants(variants, {"regular": regular_map, "filled": filled_map}, "regular")
    return variants, icons


def generate_ionicons() -> Tuple[List[dict], List[dict]]:
    regular_path = ttf_path("ionicons", "ionicons-regular.ttf")
    outline_path = ttf_path("ionicons", "ionicons-outline.ttf")
    sharp_path = ttf_path("ionicons", "ionicons-sharp.ttf")
    regular_map = {normalize_kebab(name): cp for name, cp in load_cmap(regular_path).items()}
    outline_map = {normalize_kebab(name): cp for name, cp in load_cmap(outline_path).items()}
    sharp_map = {normalize_kebab(name): cp for name, cp in load_cmap(sharp_path).items()}
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(regular_path),
            "ttf_asset_path": str(regular_path).replace("\\", "/"),
        },
        {
            "id": "outline",
            "style": "Outline",
            "size": "Regular",
            "family": load_family(outline_path),
            "ttf_asset_path": str(outline_path).replace("\\", "/"),
        },
        {
            "id": "sharp",
            "style": "Sharp",
            "size": "Regular",
            "family": load_family(sharp_path),
            "ttf_asset_path": str(sharp_path).replace("\\", "/"),
        },
    ]
    icons = merge_variants(
        variants,
        {"regular": regular_map, "outline": outline_map, "sharp": sharp_map},
        "regular",
    )
    return variants, icons


def generate_lobe() -> Tuple[List[dict], List[dict]]:
    ttf = ttf_path("lobe", "lobe-regular.ttf")
    icon_map = {normalize_kebab(name): cp for name, cp in load_cmap(ttf).items()}
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(ttf),
            "ttf_asset_path": str(ttf).replace("\\", "/"),
        }
    ]
    icons = merge_variants(variants, {"regular": icon_map}, "regular")
    return variants, icons


def generate_lucide() -> Tuple[List[dict], List[dict]]:
    ttf = ttf_path("lucide", "lucide-regular.ttf")
    icon_map = {normalize_kebab(name): cp for name, cp in load_cmap(ttf).items()}
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(ttf),
            "ttf_asset_path": str(ttf).replace("\\", "/"),
        }
    ]
    icons = merge_variants(variants, {"regular": icon_map}, "regular")
    return variants, icons


def generate_octicons() -> Tuple[List[dict], List[dict]]:
    regular_path = ttf_path("octicons", "octicons-regular.ttf")
    tiny_path = ttf_path("octicons", "octicons-tiny.ttf")

    def parse_octicons(path: Path, size_filter: int) -> Tuple[Dict[str, int], Dict[str, int]]:
        regular: Dict[str, int] = {}
        filled: Dict[str, int] = {}
        for name, cp in load_cmap(path).items():
            match = re.match(r"^(?P<base>.+?)(?P<fill>-fill)?-(?P<size>\d+)$", name)
            if not match:
                continue
            if int(match.group("size")) != size_filter:
                continue
            base = normalize_kebab(match.group("base"))
            if match.group("fill"):
                filled[base] = cp
            else:
                regular[base] = cp
        return regular, filled

    regular_map, filled_map = parse_octicons(regular_path, 24)
    tiny_regular, tiny_filled = parse_octicons(tiny_path, 16)

    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(regular_path),
            "ttf_asset_path": str(regular_path).replace("\\", "/"),
        },
        {
            "id": "filled",
            "style": "Filled",
            "size": "Regular",
            "family": load_family(regular_path),
            "ttf_asset_path": str(regular_path).replace("\\", "/"),
        },
        {
            "id": "tiny",
            "style": "Regular",
            "size": "Tiny",
            "family": load_family(tiny_path),
            "ttf_asset_path": str(tiny_path).replace("\\", "/"),
            "feature": "octicons-tiny",
        },
        {
            "id": "tiny-filled",
            "style": "Filled",
            "size": "Tiny",
            "family": load_family(tiny_path),
            "ttf_asset_path": str(tiny_path).replace("\\", "/"),
            "feature": "octicons-tiny",
        },
    ]
    icons = merge_variants(
        variants,
        {
            "regular": regular_map,
            "filled": filled_map,
            "tiny": tiny_regular,
            "tiny-filled": tiny_filled,
        },
        "regular",
    )
    return variants, icons


def generate_phosphor() -> Tuple[List[dict], List[dict]]:
    style_dirs = {
        "regular": ("Regular", "phosphor-regular.ttf"),
        "fill": ("Filled", "phosphor-filled.ttf"),
        "bold": ("Bold", "phosphor-bold.ttf"),
        "duotone": ("Duotone", "phosphor-duotone.ttf"),
        "light": ("Light", "phosphor-light.ttf"),
        "thin": ("Thin", "phosphor-thin.ttf"),
    }
    variants: List[dict] = []
    variant_maps: Dict[str, Dict[str, int]] = {}
    for style_id, (style_name, filename) in style_dirs.items():
        ttf = ttf_path("phosphor", filename)
        selection_path = Path(f"tp/phosphor-web/src/{style_id}/selection.json")
        icon_map = load_icomoon_selection(selection_path)
        variants.append(
            {
                "id": style_id,
                "style": style_name,
                "size": "Regular",
                "family": load_family(ttf),
                "ttf_asset_path": str(ttf).replace("\\", "/"),
            }
        )
        variant_maps[style_id] = icon_map
    icons = merge_variants(variants, variant_maps, "regular")
    return variants, icons


def generate_remixicon() -> Tuple[List[dict], List[dict]]:
    ttf = ttf_path("remixicon", "remixicon-regular.ttf")
    outline, filled = parse_remixicon_glyphs(Path("tp/RemixIcon/fonts/remixicon.glyph.json"))
    variants = [
        {
            "id": "outline",
            "style": "Outline",
            "size": "Regular",
            "family": load_family(ttf),
            "ttf_asset_path": str(ttf).replace("\\", "/"),
        },
        {
            "id": "filled",
            "style": "Filled",
            "size": "Regular",
            "family": load_family(ttf),
            "ttf_asset_path": str(ttf).replace("\\", "/"),
        },
    ]
    icons = merge_variants(variants, {"outline": outline, "filled": filled}, "outline")
    return variants, icons


def generate_tabler() -> Tuple[List[dict], List[dict]]:
    regular_path = ttf_path("tabler", "tabler-regular.ttf")
    filled_path = ttf_path("tabler", "tabler-filled.ttf")
    regular_map = {normalize_kebab(name): cp for name, cp in load_cmap(regular_path).items()}
    filled_map = {normalize_kebab(name): cp for name, cp in load_cmap(filled_path).items()}
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(regular_path),
            "ttf_asset_path": str(regular_path).replace("\\", "/"),
        },
        {
            "id": "filled",
            "style": "Filled",
            "size": "Regular",
            "family": load_family(filled_path),
            "ttf_asset_path": str(filled_path).replace("\\", "/"),
        },
    ]
    icons = merge_variants(variants, {"regular": regular_map, "filled": filled_map}, "regular")
    return variants, icons


def generate_simple(pack_id: str, filename: str) -> Tuple[List[dict], List[dict]]:
    ttf = ttf_path(pack_id, filename)
    icon_map = {normalize_kebab(name): cp for name, cp in load_cmap(ttf).items()}
    variants = [
        {
            "id": "regular",
            "style": "Regular",
            "size": "Regular",
            "family": load_family(ttf),
            "ttf_asset_path": str(ttf).replace("\\", "/"),
        }
    ]
    icons = merge_variants(variants, {"regular": icon_map}, "regular")
    return variants, icons


def main() -> int:
    generators = {
        "bootstrap": generate_bootstrap,
        "heroicons": generate_heroicons,
        "carbon": generate_carbon,
        "devicon": generate_devicon,
        "feather": generate_feather,
        "fluentui": generate_fluentui,
        "iconoir": generate_iconoir,
        "ionicons": generate_ionicons,
        "lobe": generate_lobe,
        "lucide": generate_lucide,
        "octicons": generate_octicons,
        "phosphor": generate_phosphor,
        "remixicon": generate_remixicon,
        "tabler": generate_tabler,
    }

    for pack_id, gen in generators.items():
        variants, icons = gen()
        write_map(pack_id, variants, icons)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
