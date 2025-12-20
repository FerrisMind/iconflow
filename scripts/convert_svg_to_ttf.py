#!/usr/bin/env python3
"""Конвертация SVG-шрифта в TTF с помощью fontTools.

Сценарий читает SVG-шрифт (формат с тегами <font>/<glyph>) и
собирает TTF-файл. Поддерживаются пути d со слоями Line/Quadratic
Bezier/Cubic Bezier и Arc (дуги аппроксимируются кубиками, затем
конвертируются в квадратичные кривые). Скрипт минимален и
подходит для одноцветных иконок.
"""

from __future__ import annotations

import argparse
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Dict, Iterable, Tuple

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.pens.transformPen import TransformPen
from fontTools.ttLib import newTable
from svgpathtools import Arc, CubicBezier, Line, Path as SvgPath, QuadraticBezier, parse_path


def _parse_svg_font(svg_path: Path) -> Tuple[int, int, int, int, Iterable[Dict[str, object]]]:
    """Читает SVG-шрифт и возвращает метрики и глифы."""
    if not svg_path.exists():
        raise FileNotFoundError(f"SVG-файл не найден: {svg_path}")
    if svg_path.stat().st_size == 0:
        raise ValueError(f"SVG-файл пуст: {svg_path}")

    tree = ET.parse(svg_path)
    root = tree.getroot()
    ns = {"svg": root.tag.split("}")[0].strip("{") if "}" in root.tag else ""}

    font_el = root.find(".//svg:font", ns) if ns["svg"] else root.find(".//font")
    if font_el is None:
        raise ValueError("В файле не найден тег <font>")

    font_face = font_el.find("svg:font-face", ns) if ns["svg"] else font_el.find("font-face")
    units_per_em = int(font_face.attrib.get("units-per-em", "1000")) if font_face is not None else 1000
    ascent = int(font_face.attrib.get("ascent", str(int(units_per_em * 0.8)))) if font_face is not None else int(
        units_per_em * 0.8
    )
    descent_raw = font_face.attrib.get("descent") if font_face is not None else None
    descent = abs(int(descent_raw)) if descent_raw is not None else int(units_per_em * 0.2)
    default_advance = int(font_el.attrib.get("horiz-adv-x", str(units_per_em)))

    glyph_elements = font_el.findall("svg:glyph", ns) if ns["svg"] else font_el.findall("glyph")
    glyphs = []
    for idx, glyph_el in enumerate(glyph_elements):
        unicode_val = glyph_el.attrib.get("unicode")
        d = glyph_el.attrib.get("d")
        adv = int(glyph_el.attrib.get("horiz-adv-x", default_advance))

        if unicode_val is None:
            # Пропускаем пустые элементы без юникода
            continue

        name = glyph_el.attrib.get("glyph-name")
        if not name:
            if len(unicode_val) == 1:
                name = f"uni{ord(unicode_val):04X}"
            else:
                name = f"glyph_{idx}"

        glyphs.append({
            "name": name,
            "unicode": unicode_val,
            "path": d,
            "advance": adv,
        })

    return units_per_em, ascent, descent, default_advance, glyphs


def _draw_path_to_pen(path: SvgPath, pen: TransformPen) -> None:
    """Рисует сегменты SVG-пути в pen."""
    current_subpath_start = None
    current_point = None

    for segment in path:
        start = (segment.start.real, segment.start.imag)
        end = (segment.end.real, segment.end.imag)

        if current_point is None or start != current_point:
            # Новая под-петля
            pen.moveTo(start)
            current_subpath_start = start

        if isinstance(segment, Line):
            pen.lineTo(end)
        elif isinstance(segment, QuadraticBezier):
            pen.qCurveTo((segment.control.real, segment.control.imag), end)
        elif isinstance(segment, CubicBezier):
            pen.curveTo(
                (segment.control1.real, segment.control1.imag),
                (segment.control2.real, segment.control2.imag),
                end,
            )
        elif isinstance(segment, Arc):
            for cubic in segment.as_cubic_curves():
                pen.curveTo(
                    (cubic.control1.real, cubic.control1.imag),
                    (cubic.control2.real, cubic.control2.imag),
                    (cubic.end.real, cubic.end.imag),
                )
        else:
            raise ValueError(f"Неизвестный тип сегмента: {type(segment)}")

        current_point = end

        if current_subpath_start and end == current_subpath_start:
            pen.closePath()
            current_subpath_start = None

    if current_subpath_start is not None:
        pen.endPath()


def _build_glyph(path_data: str, upm: int, ascent: int) -> TTGlyphPen:
    """Создаёт TTGlyph из атрибута d."""
    tt_pen = TTGlyphPen(None)
    cu2qu_pen = Cu2QuPen(tt_pen, max_err=1.0, reverse_direction=False)
    transform_pen = TransformPen(cu2qu_pen, (1, 0, 0, -1, 0, ascent))
    svg_path = parse_path(path_data)
    _draw_path_to_pen(svg_path, transform_pen)
    return tt_pen.glyph()


def convert(svg_path: Path, ttf_path: Path) -> None:
    upm, ascent, descent, default_adv, glyph_entries = _parse_svg_font(svg_path)

    glyph_order = [".notdef"]
    glyphs = {".notdef": TTGlyphPen(None).glyph()}
    h_metrics: Dict[str, Tuple[int, int]] = {".notdef": (default_adv, 0)}
    cmap = {}

    for entry in glyph_entries:
        path_data = entry["path"]
        name = entry["name"]  # type: ignore[index]
        unicode_val = entry["unicode"]  # type: ignore[index]
        advance = entry["advance"]  # type: ignore[index]

        if not path_data:
            # Разрешаем пустые глифы (например, пробел)
            glyph_obj = TTGlyphPen(None).glyph()
        else:
            glyph_obj = _build_glyph(path_data, upm, ascent)

        glyph_order.append(name)
        glyphs[name] = glyph_obj
        h_metrics[name] = (advance, 0)
        if len(unicode_val) == 1:
            cmap[ord(unicode_val)] = name

    fb = FontBuilder(upm, isTTF=True)
    fb.setupGlyphOrder(glyph_order)
    fb.setupCharacterMap(cmap)
    fb.setupGlyf(glyphs)
    fb.setupHorizontalMetrics(h_metrics)
    fb.setupHorizontalHeader(ascent=ascent, descent=-descent)
    fb.setupOS2(
        sTypoAscender=ascent,
        sTypoDescender=-descent,
        sTypoLineGap=0,
        usWinAscent=ascent,
        usWinDescent=descent,
    )
    fb.setupNameTable({
        "familyName": "Lucide",
        "styleName": "Regular",
        "uniqueFontIdentifier": "Lucide-Regular",
        "fullName": "Lucide Regular",
        "psName": "Lucide-Regular",
        "version": "1.0",
    })
    fb.setupPost()
    fb.setupMaxp()

    ttf_path.parent.mkdir(parents=True, exist_ok=True)
    fb.save(str(ttf_path))


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Конвертация SVG-шрифта в TTF")
    parser.add_argument(
        "--src",
        type=Path,
        default=Path("c:/iconflow/tp/lucide/lucide-font/lucide.svg"),
        help="Путь к исходному SVG-шрифту",
    )
    parser.add_argument(
        "--dst",
        type=Path,
        default=Path("c:/iconflow/tp/lucide/lucide-font/lucide.from-svg.ttf"),
        help="Куда сохранить TTF",
    )
    args = parser.parse_args(argv)

    try:
        convert(args.src, args.dst)
    except Exception as exc:  # noqa: BLE001
        print(f"Ошибка конвертации: {exc}", file=sys.stderr)
        return 1

    print(f"Готово: {args.dst}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))







