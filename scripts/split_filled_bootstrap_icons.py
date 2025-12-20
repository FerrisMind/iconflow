#!/usr/bin/env python3
"""Перенос SVG-иконок Bootstrap с заливкой (fill) в отдельную папку.

Каталог входа: tp/bootstrap/icons

Правило: берём все *.svg, у которых внутри файла есть строка с "fill="
(без учёта регистра), и перемещаем их в подпапку `filled` внутри
основного каталога icons.

Запуск из корня репозитория `C:\iconflow`:

    python split_filled_bootstrap_icons.py
"""

from __future__ import annotations

from pathlib import Path


ICONS_DIR = Path("tp/bootstrap/icons/filled")
TARGET_SUBDIR = "filled"


def main() -> int:
    root = Path.cwd()
    icons_dir = root / ICONS_DIR

    if not icons_dir.exists():
        print(f"Каталог с иконками не найден: {icons_dir}")
        return 1

    svg_files = sorted(p for p in icons_dir.glob("*.svg") if p.is_file())
    if not svg_files:
        print(f"В {icons_dir} нет *.svg")
        return 0

    # Перемещаем ТОЛЬКО иконки, у которых имя заканчивается на '-fill.svg'
    # (например, '2-square-fill.svg'), остальные остаются на месте.
    targets = [p for p in svg_files if p.stem.endswith("-fill")]

    if not targets:
        print(f"В {icons_dir} не найдено SVG с атрибутом fill.")
        return 0

    target_dir = icons_dir / TARGET_SUBDIR
    target_dir.mkdir(parents=True, exist_ok=True)

    print(
        f"Найдено {len(targets)} SVG с 'fill', перемещаем в "
        f"{target_dir.relative_to(root)}:\n"
    )

    for path in targets:
        dest = target_dir / path.name
        print(f"  {path.name} -> {dest.relative_to(root)}")
        try:
            path.rename(dest)
        except OSError as exc:  # noqa: BLE001
            print(f"    ! ошибка перемещения: {exc}")

    print("\nГотово.")
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())

