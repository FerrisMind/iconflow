#!/usr/bin/env python3
"""Конвертация шрифтов WOFF/WOFF2 в TTF.

Примеры запуска из `C:\iconflow`:

    python woff_to_ttf.py path/to/font.woff
    python woff_to_ttf.py path/to/font.woff2 path/to/out.ttf

Если выходной путь не указан, создаётся файл рядом с исходным
с тем же именем и расширением `.ttf`.

Требования:
  pip install fonttools brotli
"""

from __future__ import annotations

import sys
from pathlib import Path

from fontTools.ttLib import TTFont


def convert_woff_to_ttf(src: Path, dst: Path) -> None:
    if not src.exists():
        raise FileNotFoundError(f"Исходный файл не найден: {src}")

    if src.suffix.lower() not in {".woff", ".woff2"}:
        raise ValueError(f"Ожидался .woff или .woff2, получено: {src.suffix}")

    print(f"Читаю:  {src}")
    font = TTFont(str(src))
    print(f"Пишу:   {dst}")
    font.flavor = None  # сбросить WOFF/WOFF2-обёртку
    font.save(str(dst))
    print("Готово.")


def main(argv: list[str]) -> int:
    if not argv or len(argv) > 2:
        print("Использование: python woff_to_ttf.py <src.woff/.woff2> [dst.ttf]")
        return 1

    src = Path(argv[0]).resolve()
    if len(argv) == 2:
        dst = Path(argv[1]).resolve()
    else:
        dst = src.with_suffix(".ttf")

    try:
        convert_woff_to_ttf(src, dst)
    except Exception as exc:  # noqa: BLE001
        print(f"Ошибка: {exc}")
        return 1

    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main(sys.argv[1:]))

