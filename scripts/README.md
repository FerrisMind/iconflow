# Scripts for iconflow

This directory contains helper scripts for generating, validating, and processing icons and fonts in the iconflow project.

## Validation and verification scripts

### `validate_assets.py`
**Purpose:** Validates icon JSON maps (`assets/maps/*.json`) against the schema and checks data correctness.

**What it checks:**
- JSON files conform to `assets/schema/iconflow-pack.schema.json`
- Uniqueness of `variant.id` and `icon.name`
- Existence of TTF files specified in variants
- Matching `family` in JSON and actual font name in TTF
- Presence of feature flags in `Cargo.toml` for variants with `feature`
- Absence of surrogate codepoints (0xD800-0xDFFF)
- Correctness of `overrides` and `availability` for icons

**Usage:**
```bash
python scripts/validate_assets.py
```

**Dependencies:** `jsonschema`, `fonttools`, `tomllib`/`tomli`

---

### `verify_integrity.py`
**Purpose:** Verifies TTF file integrity by comparing SHA256 hashes with a manifest.

**What it does:**
- Reads manifest `ASSETS_MANIFEST.json`
- Finds all TTF files in `assets/`
- Computes SHA256 for each file
- Compares with hashes from manifest
- Detects missing, extra, or modified files

**Usage:**
```bash
python scripts/verify_integrity.py
```

**Dependencies:** Python standard library

---

### `check_fonts.py`
**Purpose:** Checks for TTF files in icon pack directories and copies them to `assets/fonts/`.

**What it does:**
- Recursively searches for all TTF files in `tp/`
- Checks font availability for each icon pack
- Copies found TTF files to `assets/fonts/` with unique names
- Outputs statistics for icon packs

**Usage:**
```bash
python scripts/check_fonts.py
```

**Dependencies:** Python standard library

---

## Generation scripts

### `map_gen.py`
**Purpose:** Generates icon JSON maps (`assets/maps/*.json`) from TTF fonts.

**What it does:**
- Reads TTF files from `assets/fonts/`
- Extracts cmap (codepoint → glyph name mapping)
- Normalizes icon names to kebab-case
- Merges variants (regular/filled/outline, etc.) into a unified structure
- Generates JSON with variants and icons
- Creates files in `assets/maps/` for each icon pack

**Usage:**
```bash
python scripts/map_gen.py
```

**Dependencies:** `fonttools`

---

### `svg_to_rust.py`
**Purpose:** Generates Rust code from SVG icon files, creating constants with SVG path data instead of binary font files.

**What it does:**
- Reads SVG files from a specified directory
- Extracts path data from SVG elements (path, polyline, polygon, circle, rect, line)
- Converts icon names to Rust identifiers (PascalCase)
- Generates Rust file with constants for each icon
- Creates helper functions to access icons by name

**Usage:**
```bash
# Basic usage
python scripts/svg_to_rust.py path/to/svg/icons -n "PackName" -o output.rs

# With custom pattern
python scripts/svg_to_rust.py path/to/svg/icons -n "PackName" -o output.rs -p "*.svg"
```

**Example:**
```bash
python scripts/svg_to_rust.py tp/feather/icons -n "Feather" -o src/generated/feather_svg.rs
```

**Output format:**
- Each icon becomes a Rust constant: `pub const IconName: &str = "path data";`
- Icons with multiple paths become arrays: `pub const IconName: &[&str] = &[...];`
- Helper function `get_icon_path(name)` to lookup icons by kebab-case name
- Constant `ICON_NAMES` with list of all available icon names

**Dependencies:** Python standard library (`xml.etree.ElementTree`)

---

## Font processing scripts

### `patch_names.py`
**Purpose:** Patches name table in TTF files to set unique font family names.

**What it does:**
- Reads TTF files
- Sets unique family names (e.g., "Bootstrap Regular", "Bootstrap Filled")
- Updates PostScript name, Full name, Preferred Family, and other name table fields
- Supports automatic name inference from directory structure or manual specification

**Usage:**
```bash
# Automatically process all fonts in assets/fonts/
python scripts/patch_names.py --apply-defaults

# Process a specific file
python scripts/patch_names.py --file path/to/font.ttf --family "Custom Font Name"

# Process a list from JSON
python scripts/patch_names.py --targets targets.json
```

**Dependencies:** `fonttools`

---

### `woff_to_ttf.py`
**Purpose:** Converts fonts from WOFF/WOFF2 format to TTF.

**What it does:**
- Reads WOFF or WOFF2 file
- Removes WOFF wrapper
- Saves as TTF

**Usage:**
```bash
python scripts/woff_to_ttf.py input.woff [output.ttf]
```

**Dependencies:** `fonttools`, `brotli` (for WOFF2)

---

## SVG icon processing scripts

### `split_filled_bootstrap_icons.py`
**Purpose:** Moves Bootstrap icons with fill into a separate subdirectory.

**What it does:**
- Finds all SVG files in `tp/bootstrap/icons/filled/`
- Moves files ending with `-fill.svg` to `filled/` subdirectory

**Usage:**
```bash
python scripts/split_filled_bootstrap_icons.py
```

**Dependencies:** Python standard library

---

### `split_icons_by_style.py`
**Purpose:** Organizes SVG icons by style (outline/sharp/solid/filled/glyph/regular).

**What it does:**
- Analyzes SVG file names
- Determines style by keywords in the name (outline, sharp, solid, filled, glyph)
- Moves icons to corresponding subdirectories

**Usage:**
```bash
# By default processes tp/bootstrap/icons
python scripts/split_icons_by_style.py

# Specify a different directory
python scripts/split_icons_by_style.py --src-dir tp/some-pack/icons
```

**Dependencies:** Python standard library

---

### `move_16_icons.py`
**Purpose:** Moves Octicons icons of size 16px to a separate subdirectory.

**What it does:**
- Finds all SVG files in `tp/octicons/icons/`
- Moves files whose names end with `16` to `16/` subdirectory

**Usage:**
```bash
python scripts/move_16_icons.py
```

**Dependencies:** Python standard library

---

### `remove_color_icons.py`
**Purpose:** Removes color icons from lobe-icons.

**What it does:**
- Finds all SVG files with `color` substring in the name in `tp/lobe-icons/packages/static-svg/icons/`
- Deletes found files

**Usage:**
```bash
python scripts/remove_color_icons.py
```

**Dependencies:** Python standard library

---

### `expand_strokes_with_inkscape.py`
**Purpose:** Converts stroke to path for SVG icons using Inkscape.

**What it does:**
- Processes all SVG files in the specified directory
- Uses Inkscape CLI to convert stroke → path
- Saves results to `*-expanded/` directory
- Supports parallel processing

**Usage:**
```bash
python scripts/expand_strokes_with_inkscape.py [--src-dir path/to/icons] [--inkscape path/to/inkscape]
```

**Dependencies:** `inkscape` (must be in PATH or specified via `--inkscape`)

---

## SVG → TTF conversion scripts

### `convert_svg_to_ttf.py`
**Purpose:** Converts SVG font (format with `<font>`/`<glyph>` tags) to TTF using fontTools.

**What it does:**
- Reads SVG font (not individual SVG icons, but an SVG file with `<font>` tags)
- Parses paths (d) and converts to glyphs
- Approximates arcs with cubic curves, then converts to quadratic curves
- Creates TTF file

**Usage:**
```bash
python scripts/convert_svg_to_ttf.py input.svg output.ttf
```

**Dependencies:** `fonttools`, `svgpathtools`

---

### `svg_pack_to_ttf_fontforge.py`
**Purpose:** Builds TTF font from a directory of individual SVG icons using FontForge.

**What it does:**
- Reads all SVG files from the specified directory
- Assigns sequential codepoints starting from U+E000
- Creates TTF font using FontForge Python API
- Supports using pre-expanded icons (from `*-expanded/`)

**Usage:**
```bash
fontforge -script scripts/svg_pack_to_ttf_fontforge.py \
    --src-dir path/to/icons \
    --dst-font output.ttf \
    --family "Font Family Name"
```

**Dependencies:** `fontforge` with Python API

---

### `gen_fontforge_preview_html.py`
**Purpose:** Generates HTML preview for checking TTF font created via FontForge.

**What it does:**
- Reads TTF file and source SVG icons
- Creates HTML page with icon grid
- Shows glyph from font alongside source SVG for visual comparison
- Displays codepoint and icon name

**Usage:**
```bash
python scripts/gen_fontforge_preview_html.py
```

**Dependencies:** Python standard library

**Note:** The script contains hardcoded paths and may require editing.

---

## Common dependencies

Most scripts require:

```bash
pip install fonttools jsonschema tomli  # or tomllib (Python 3.11+)
```

For WOFF2 support:
```bash
pip install brotli
```

For SVG path support:
```bash
pip install svgpathtools
```

For FontForge support:
- Install FontForge with Python API support
- Use `fontforge -script` to run scripts

---

## Typical workflow

1. **Prepare source data:**
   - Use `check_fonts.py` to find and copy TTF files
   - Use `woff_to_ttf.py` to convert WOFF/WOFF2 to TTF

2. **Process fonts:**
   - Use `patch_names.py` to set unique family names

3. **Generate maps:**
   - Use `map_gen.py` to create JSON maps from TTF

4. **Validate:**
   - Use `validate_assets.py` to check map correctness
   - Use `verify_integrity.py` to verify file integrity

5. **Process SVG (if needed):**
   - Use `split_icons_by_style.py` to organize icons
   - Use `expand_strokes_with_inkscape.py` to prepare for conversion
   - Use `svg_pack_to_ttf_fontforge.py` to create TTF from SVG
