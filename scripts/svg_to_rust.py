#!/usr/bin/env python3
"""
Generate Rust code from SVG icon files.

This script reads SVG files from a directory and generates a Rust file where
each icon is represented as a constant containing SVG path data.
"""

from __future__ import annotations

import argparse
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Dict, List, Tuple


def normalize_rust_name(name: str) -> str:
    """Convert icon name to Rust identifier (PascalCase)."""
    # Remove file extension
    name = name.rsplit(".", 1)[0] if "." in name else name
    # Split by common separators
    parts = re.split(r"[-_\s]+", name)
    # Convert to PascalCase
    return "".join(word.capitalize() for word in parts if word)


def normalize_kebab(name: str) -> str:
    """Normalize icon name to kebab-case."""
    cleaned = name.strip().lower().replace("_", "-")
    cleaned = re.sub(r"[^a-z0-9-]+", "-", cleaned)
    cleaned = re.sub(r"-{2,}", "-", cleaned)
    cleaned = cleaned.strip("-")
    return cleaned


def extract_svg_paths(svg_content: str) -> List[str]:
    """
    Extract all path data from SVG content.
    Returns list of path 'd' attribute values and other shape data.
    """
    try:
        root = ET.fromstring(svg_content)
    except ET.ParseError as e:
        raise ValueError(f"Failed to parse SVG: {e}") from e

    # Register namespaces
    namespaces = {
        "svg": "http://www.w3.org/2000/svg",
        "": "http://www.w3.org/2000/svg",
    }

    paths = []
    
    # Find all path elements
    for path in root.findall(".//path", namespaces):
        d_attr = path.get("d")
        if d_attr:
            paths.append(d_attr)
    
    # Handle polyline elements - convert points to path-like format
    for polyline in root.findall(".//polyline", namespaces):
        points = polyline.get("points")
        if points:
            # Convert polyline points to path data format
            # Format: "x1,y1 x2,y2 ..." -> "M x1,y1 L x2,y2 ..."
            coords = points.strip().replace(",", " ").split()
            if len(coords) >= 2:
                path_data = f"M {coords[0]},{coords[1]}"
                for i in range(2, len(coords), 2):
                    if i + 1 < len(coords):
                        path_data += f" L {coords[i]},{coords[i+1]}"
                paths.append(path_data)
    
    # Handle polygon elements - similar to polyline but closed
    for polygon in root.findall(".//polygon", namespaces):
        points = polygon.get("points")
        if points:
            coords = points.strip().replace(",", " ").split()
            if len(coords) >= 2:
                path_data = f"M {coords[0]},{coords[1]}"
                for i in range(2, len(coords), 2):
                    if i + 1 < len(coords):
                        path_data += f" L {coords[i]},{coords[i+1]}"
                path_data += " Z"  # Close the path
                paths.append(path_data)
    
    # Handle circle elements
    for circle in root.findall(".//circle", namespaces):
        cx = circle.get("cx", "0")
        cy = circle.get("cy", "0")
        r = circle.get("r", "0")
        # Convert circle to path: M cx-r,cy A r,r 0 1,1 cx+r,cy A r,r 0 1,1 cx-r,cy
        path_data = f"M {float(cx)-float(r)},{cy} A {r},{r} 0 1,1 {float(cx)+float(r)},{cy} A {r},{r} 0 1,1 {float(cx)-float(r)},{cy}"
        paths.append(path_data)
    
    # Handle rect elements
    for rect in root.findall(".//rect", namespaces):
        x = rect.get("x", "0")
        y = rect.get("y", "0")
        width = rect.get("width", "0")
        height = rect.get("height", "0")
        rx = rect.get("rx", "0")
        ry = rect.get("ry", rx)  # Use rx if ry not specified
        
        if float(rx) == 0 and float(ry) == 0:
            # Simple rectangle
            path_data = f"M {x},{y} L {float(x)+float(width)},{y} L {float(x)+float(width)},{float(y)+float(height)} L {x},{float(y)+float(height)} Z"
        else:
            # Rounded rectangle - simplified approximation
            path_data = f"M {float(x)+float(rx)},{y} L {float(x)+float(width)-float(rx)},{y} A {rx},{ry} 0 0,1 {float(x)+float(width)},{float(y)+float(ry)} L {float(x)+float(width)},{float(y)+float(height)-float(ry)} A {rx},{ry} 0 0,1 {float(x)+float(width)-float(rx)},{float(y)+float(height)} L {float(x)+float(rx)},{float(y)+float(height)} A {rx},{ry} 0 0,1 {x},{float(y)+float(height)-float(ry)} L {x},{float(y)+float(ry)} A {rx},{ry} 0 0,1 {float(x)+float(rx)},{y} Z"
        paths.append(path_data)
    
    # Handle line elements
    for line in root.findall(".//line", namespaces):
        x1 = line.get("x1", "0")
        y1 = line.get("y1", "0")
        x2 = line.get("x2", "0")
        y2 = line.get("y2", "0")
        path_data = f"M {x1},{y1} L {x2},{y2}"
        paths.append(path_data)

    return paths


def extract_svg_content(svg_path: Path) -> str:
    """Extract SVG content, handling different SVG formats."""
    content = svg_path.read_text(encoding="utf-8")
    
    # Remove XML declaration if present
    content = re.sub(r'<\?xml[^>]*\?>', '', content)
    
    # Extract viewBox and other attributes if needed
    return content.strip()


def extract_path_data(svg_path: Path) -> Tuple[str, List[str]]:
    """
    Extract path data from SVG file.
    Returns (icon_name, list_of_path_data).
    """
    content = extract_svg_content(svg_path)
    paths = extract_svg_paths(content)
    
    if not paths:
        raise ValueError(f"No path data found in {svg_path}")
    
    icon_name = normalize_kebab(svg_path.stem)
    return icon_name, paths


def escape_rust_string(s: str) -> str:
    """Escape string for Rust string literal."""
    return s.replace('\\', '\\\\').replace('"', '\\"').replace('\n', '\\n')


def generate_rust_constant(icon_name: str, rust_name: str, paths: List[str]) -> str:
    """Generate Rust constant for an icon."""
    if len(paths) == 1:
        # Single path - simple constant
        path_data = escape_rust_string(paths[0])
        return f'pub const {rust_name}: &str = "{path_data}";'
    else:
        # Multiple paths - array constant
        path_strings = [escape_rust_string(p) for p in paths]
        paths_str = ",\n        ".join(f'"{p}"' for p in path_strings)
        return f"""pub const {rust_name}: &[&str] = &[
        {paths_str}
    ];"""


def generate_rust_file(
    pack_name: str,
    icons: Dict[str, Tuple[str, List[str]]],
    output_path: Path,
) -> None:
    """
    Generate Rust file with icon constants.
    
    Args:
        pack_name: Name of the icon pack (used for module name)
        icons: Dict mapping icon_name -> (rust_name, paths)
        output_path: Path to output Rust file
    """
    rust_module_name = pack_name.lower().replace("-", "_")
    
    lines = [
        "// @generated by scripts/svg_to_rust.py. DO NOT EDIT.",
        "",
        f"// Icon pack: {pack_name}",
        f"// Total icons: {len(icons)}",
        "",
    ]
    
    # Generate constants for each icon
    sorted_icons = sorted(icons.items(), key=lambda x: x[0])
    
    for icon_name, (rust_name, paths) in sorted_icons:
        lines.append(generate_rust_constant(icon_name, rust_name, paths))
        lines.append("")
    
    # Generate helper functions/structs if needed
    lines.extend([
        "",
        "/// Get SVG path data for an icon by name (kebab-case).",
        "pub fn get_icon_path(name: &str) -> Option<&'static [&'static str]> {",
        "    match name {",
    ])
    
    for icon_name, (rust_name, paths) in sorted_icons:
        if len(paths) == 1:
            lines.append(f'        "{icon_name}" => Some(&[{rust_name}]),')
        else:
            lines.append(f'        "{icon_name}" => Some({rust_name}),')
    
    lines.extend([
        "        _ => None,",
        "    }",
        "}",
        "",
        "/// List all available icon names.",
        "pub const ICON_NAMES: &[&str] = &[",
    ])
    
    for icon_name, _ in sorted_icons:
        lines.append(f'    "{icon_name}",')
    
    lines.extend([
        "];",
        "",
    ])
    
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {output_path} with {len(icons)} icons")


def process_svg_directory(
    svg_dir: Path,
    pack_name: str,
    output_path: Path,
    pattern: str = "*.svg",
) -> None:
    """
    Process directory of SVG files and generate Rust code.
    
    Args:
        svg_dir: Directory containing SVG files
        pack_name: Name of the icon pack
        output_path: Path to output Rust file
        pattern: Glob pattern for SVG files (default: "*.svg")
    """
    if not svg_dir.is_dir():
        raise ValueError(f"Not a directory: {svg_dir}")
    
    svg_files = sorted(svg_dir.glob(pattern))
    
    if not svg_files:
        raise ValueError(f"No SVG files found in {svg_dir} matching pattern {pattern}")
    
    icons: Dict[str, Tuple[str, List[str]]] = {}
    errors: List[str] = []
    
    for svg_file in svg_files:
        try:
            icon_name, paths = extract_path_data(svg_file)
            rust_name = normalize_rust_name(svg_file.stem)
            
            if icon_name in icons:
                print(f"Warning: Duplicate icon name '{icon_name}' from {svg_file}", file=sys.stderr)
                continue
            
            icons[icon_name] = (rust_name, paths)
        except Exception as e:
            errors.append(f"{svg_file}: {e}")
    
    if errors:
        print("Errors encountered:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        if not icons:
            raise ValueError("No valid icons found")
    
    generate_rust_file(pack_name, icons, output_path)


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate Rust code from SVG icon files"
    )
    parser.add_argument(
        "svg_dir",
        type=Path,
        help="Directory containing SVG icon files",
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        required=True,
        help="Output Rust file path",
    )
    parser.add_argument(
        "-n", "--name",
        required=True,
        help="Icon pack name",
    )
    parser.add_argument(
        "-p", "--pattern",
        default="*.svg",
        help="Glob pattern for SVG files (default: *.svg)",
    )
    
    args = parser.parse_args()
    
    try:
        process_svg_directory(
            args.svg_dir,
            args.name,
            args.output,
            args.pattern,
        )
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())

