#!/usr/bin/env python3
"""Сборка TTF-шрифта из папки SVG-иконок через FontForge.

Скрипт предполагает установленный FontForge c Python API.
Обычно его запускают так:

    fontforge -script svg_pack_to_ttf_fontforge.py \
        --src-dir path/to/icons \
        --dst-font out/font.ttf

По умолчанию используется пакет lucide в этом репозитории.
"""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Iterable

import fontforge  # type: ignore[import]


def iter_svg_files(src_dir: Path) -> Iterable[Path]:
    """Возвращает SVG-файлы из директории, отсортированные по имени."""
    for entry in sorted(src_dir.glob("*.svg")):
        if entry.is_file():
            yield entry


def build_font_from_svgs(src_dir: Path, dst_font: Path, family: str, start_codepoint: int = 0xE000) -> None:
    """Создаёт TTF-шрифт из SVG-иконок.

    Каждой иконке назначается последовательный код из приватного диапазона
    (по умолчанию U+E000, U+E001, ...). Имя глифа = имя файла без расширения.
    """

    if not src_dir.exists():
        raise FileNotFoundError(f"Директория с иконками не найдена: {src_dir}")

    # Если есть предварительно расширенные иконки (expand_strokes_with_inkscape.py),
    # используем их. В противном случае берём оригинальные SVG.
    expanded_dir = src_dir.parent / f"{src_dir.name}-expanded"
    search_dir = expanded_dir if expanded_dir.exists() else src_dir

    svg_files = list(iter_svg_files(search_dir))
    if not svg_files:
        raise ValueError(f"В директории {src_dir} нет SVG-файлов")

    font = fontforge.font()
    font.encoding = "UnicodeFull"
    font.familyname = family
    font.fontname = f"{family.replace(' ', '')}-Regular"
    font.fullname = f"{family} Regular"
    font.weight = "Regular"

    # Базовые метрики
    em = 1000
    font.em = em
    font.ascent = int(em * 0.8)
    font.descent = em - font.ascent

    codepoint = start_codepoint

    for svg_path in svg_files:
        name = svg_path.stem
        # Создаём глиф с заданным юникодом
        glyph = font.createChar(codepoint, name)
        glyph.importOutlines(str(svg_path))

        # Приводим контуры в порядок до расчёта метрик:
        #  - объединяем пересекающиеся контуры (иначе могут появляться «дырки»,
        #    как в перемычке буквы A у a-arrow-down)
        #  - выправляем направление и округляем координаты.
        glyph.removeOverlap()
        glyph.correctDirection()
        glyph.round()

        # Нормализация ширины: небольшая прибавка к ширине контура
        glyph.left_side_bearing = 10
        glyph.right_side_bearing = 10
        xmin, _, xmax, _ = glyph.boundingBox()
        # boundingBox() может вернуть float, приводим к int
        width = int(xmax - xmin) + int(glyph.right_side_bearing)
        # На всякий случай защищаемся от нулевой/отрицательной ширины
        glyph.width = max(width, int(em * 0.6))

        codepoint += 1

    dst_font.parent.mkdir(parents=True, exist_ok=True)
    font.generate(str(dst_font))


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Собрать TTF из папки SVG-иконок через FontForge")
    parser.add_argument(
        "--src-dir",
        type=Path,
        default=Path("c:/iconflow/tp/lucide/icons"),
        help="Путь к папке с SVG-иконками",
    )
    parser.add_argument(
        "--dst-font",
        type=Path,
        default=Path("c:/iconflow/tp/lucide/lucide-font/lucide.fontforge.ttf"),
        help="Путь к выходному TTF-шрифту",
    )
    parser.add_argument(
        "--family",
        type=str,
        default="Lucide",
        help="Имя семейства шрифта",
    )
    parser.add_argument(
        "--start-codepoint",
        type=lambda s: int(s, 0),
        default=0xE000,
        help="Начальный юникод (по умолчанию 0xE000, приватная область)",
    )

    args = parser.parse_args(argv)

    build_font_from_svgs(args.src_dir, args.dst_font, args.family, args.start_codepoint)
    print(f"Готово: {args.dst_font}")
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())





