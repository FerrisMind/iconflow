#!/usr/bin/env python3
"""Перемещает иконки, заканчивающиеся на '16', в отдельную папку.

Каталог: tp/octicons/icons

Правило: все файлы, у которых имя заканчивается на '16' (перед расширением),
перемещаются в подпапку '16' внутри основного каталога.

Примеры:
- icon-16.svg -> icons/16/icon-16.svg
- something16.svg -> icons/16/something16.svg
- icon-24.svg -> остаётся в icons/

Запуск из корня репозитория `C:\iconflow`:

    python move_16_icons.py
"""

from __future__ import annotations

from pathlib import Path


ICONS_DIR = Path("tp/octicons/icons")
TARGET_SUBDIR = "16"


def main() -> int:
    root = Path.cwd()
    icons_dir = root / ICONS_DIR

    if not icons_dir.exists():
        print(f"Каталог с иконками не найден: {icons_dir}")
        return 1

    # Ищем все файлы, заканчивающиеся на '16' перед расширением
    targets = []
    for path in icons_dir.glob("*.svg"):
        if not path.is_file():
            continue
        # Проверяем, что имя заканчивается на '16' перед расширением
        stem = path.stem
        if stem.endswith("16"):
            targets.append(path)

    if not targets:
        print(f"В {icons_dir} не найдено иконок, заканчивающихся на '16'.")
        return 0

    # Создаём целевую подпапку
    target_dir = icons_dir / TARGET_SUBDIR
    target_dir.mkdir(parents=True, exist_ok=True)

    print(f"Найдено {len(targets)} иконок, заканчивающихся на '16', перемещаем в {target_dir.relative_to(root)}:\n")
    for path in sorted(targets):
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

