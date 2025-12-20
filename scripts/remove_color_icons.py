#!/usr/bin/env python3
"""Удаляет иконки с припиской `color` из lobe-icons static-svg.

Каталог: tp/lobe-icons/packages/static-svg/icons

Правило: удаляются все файлы, у которых в имени есть подстрока `color`
перед расширением, например:
- adobe-color.svg
- something-color-2.svg

Запуск из корня репозитория `C:\iconflow`:

    python remove_color_icons.py
"""

from __future__ import annotations

from pathlib import Path


ICONS_DIR = Path("tp/lobe-icons/packages/static-svg/icons")


def main() -> int:
    root = Path.cwd()
    icons_dir = root / ICONS_DIR

    if not icons_dir.exists():
        print(f"Каталог с иконками не найден: {icons_dir}")
        return 1

    targets = sorted(p for p in icons_dir.glob("*color*.svg") if p.is_file())

    if not targets:
        print(f"В {icons_dir} не найдено иконок с 'color' в имени.")
        return 0

    print(f"Найдено {len(targets)} иконок с 'color' в имени, удаляем:\n")
    for path in targets:
        print(f"  удаляю {path.name}")
        try:
            path.unlink()
        except OSError as exc:  # noqa: BLE001
            print(f"    ! ошибка удаления: {exc}")

    print("\nГотово.")
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())

