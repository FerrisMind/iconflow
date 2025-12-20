#!/usr/bin/env python3
"""Раскидывает SVG-иконки по подпапкам outline / sharp / solid / filled / glyph / regular.

По имени файла определяем стиль иконки и перемещаем её в соответствующую
подпапку внутри каталога с иконками.

Правила (по stem, без расширения):
- если имя содержит "outline" или заканчивается на "-outline"/"-outlined" -> outline
- если имя содержит "sharp" или заканчивается на "-sharp" -> sharp
- если имя содержит "solid" или заканчивается на "-solid" -> solid
- если имя содержит "glyph" или заканчивается на "-glyph" -> glyph
- если имя содержит "filled"/"fill" или заканчивается на "-fill"/"-filled" -> filled
- иначе -> regular

По умолчанию работает с каталогом `tp/bootstrap/icons`, но можно указать
любой другой через аргумент `--src-dir`.

Запуск из корня `C:\iconflow`:

    python split_icons_by_style.py
    python split_icons_by_style.py --src-dir tp/some-pack/icons
"""

from __future__ import annotations

import argparse
from pathlib import Path


def detect_style(stem: str) -> str:
    """Определяет стиль иконки по имени файла."""
    lower = stem.lower()

    if lower.endswith("-outlined") or lower.endswith("-outline") or "outline" in lower:
        return "outline"
    if lower.endswith("-sharp") or "sharp" in lower:
        return "sharp"
    if lower.endswith("-solid") or "solid" in lower:
        return "solid"
    if lower.endswith("-glyph") or "glyph" in lower:
        return "glyph"
    if (
        lower.endswith("-filled")
        or lower.endswith("-fill")
        or "filled" in lower
        or "fill" in lower
    ):
        return "filled"
    return "regular"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Разложить SVG-иконки по стилям outline/sharp/regular")
    parser.add_argument(
        "--src-dir",
        type=Path,
        default=Path("tp/bootstrap/icons"),
        help="Каталог с исходными SVG-иконками",
    )
    args = parser.parse_args(argv)

    root = Path.cwd()
    icons_dir = root / args.src_dir

    if not icons_dir.exists():
        print(f"Каталог с иконками не найден: {icons_dir}")
        return 1

    svg_files = sorted(p for p in icons_dir.glob("*.svg") if p.is_file())
    if not svg_files:
        print(f"В {icons_dir} нет *.svg")
        return 0

    # Подготовка каталога назначения
    out_dirs = {
        "outline": icons_dir / "outline",
        "sharp": icons_dir / "sharp",
        "solid": icons_dir / "solid",
        "filled": icons_dir / "filled",
        "glyph": icons_dir / "glyph",
        "regular": icons_dir / "regular",
    }
    for d in out_dirs.values():
        d.mkdir(parents=True, exist_ok=True)

    moved_counts = {key: 0 for key in out_dirs.keys()}

    print(
        f"Найдено {len(svg_files)} SVG, раскладываем по стилям "
        f"outline/sharp/solid/filled/glyph/regular внутри {icons_dir}...\n"
    )

    for path in svg_files:
        style = detect_style(path.stem)
        dest_dir = out_dirs[style]
        dest = dest_dir / path.name

        # Если файл уже в нужной подпапке, пропускаем
        if path.parent == dest_dir:
            continue

        print(f"  {path.name} -> {style}/{path.name}")
        try:
            path.rename(dest)
            moved_counts[style] += 1
        except OSError as exc:  # noqa: BLE001
            print(f"    ! ошибка перемещения: {exc}")

    print("\nИтог:")
    for style, count in moved_counts.items():
        print(f"  {style}: {count} файлов")

    print("\nГотово.")
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())

