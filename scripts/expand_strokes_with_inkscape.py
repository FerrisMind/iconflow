#!/usr/bin/env python3
"""Batch expand stroke -> path for Lucide SVG icons via Inkscape.

Скрипт проходится по всем `*.svg` в папке `tp/lucide/icons`,
для каждой иконки вызывает Inkscape CLI с `object-stroke-to-path`
и сохраняет результат в `tp/lucide/icons-expanded/` с тем же именем.

Пример запуска (из корня репо `C:/iconflow`):

    python tp/lucide/lucide-font/expand_strokes_with_inkscape.py

После успешного выполнения можно запускать сборку шрифта так, чтобы
она брала иконки из `icons-expanded`.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Iterable, Tuple


def _build_cmd(inkscape: str, src: Path, dst: Path) -> list[str]:
    return [
        inkscape,
        "--actions=select-all:all;object-stroke-to-path",
        "--export-type=svg",
        "--export-plain-svg",
        f"--export-filename={dst}",
        str(src),
    ]


def _process_one(inkscape: str, src: Path, dst: Path, force: bool) -> Tuple[Path, bool, str]:
    """Обрабатывает один SVG. Возвращает (файл, успех, stderr)."""
    if dst.exists() and not force:
        # Уже есть и не форсим — пропускаем.
        return src, True, ""

    cmd = _build_cmd(inkscape, src, dst)
    try:
        proc = subprocess.run(
            cmd,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except subprocess.CalledProcessError as exc:  # noqa: BLE001
        return src, False, exc.stderr.decode(errors="ignore") or "<пусто>"

    return src, True, ""


def main() -> int:
    parser = argparse.ArgumentParser(description="Expand stroke->path для всех SVG в каталоге через Inkscape")
    parser.add_argument(
        "--force",
        action="store_true",
        help="Перегенерировать все файлы, даже если в icons-expanded уже есть результат",
    )
    parser.add_argument(
        "--jobs",
        type=int,
        default=max(os.cpu_count() or 4, 4),
        help="Количество параллельных процессов Inkscape (по умолчанию ~= числу ядер, но не меньше 4)",
    )
    parser.add_argument(
        "--src-dir",
        type=Path,
        default=Path("c:/iconflow/tp/lucide/icons"),
        help="Каталог с исходными SVG-иконками",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=None,
        help="Каталог для сохранения SVG после expand. По умолчанию <src-dir>-expanded",
    )
    args = parser.parse_args()

    inkscape = shutil.which("inkscape")
    if inkscape is None:
        raise SystemExit(
            "Inkscape не найден в PATH. Установи Inkscape и добавь его в PATH, "
            "затем перезапусти этот скрипт."
        )

    icons_dir = args.src_dir
    out_dir = args.out_dir or (icons_dir.parent / f"{icons_dir.name}-expanded")

    if not icons_dir.exists():
        raise SystemExit(f"Папка с иконками не найдена: {icons_dir}")

    svg_files = sorted(p for p in icons_dir.glob("*.svg") if p.is_file())
    if not svg_files:
        raise SystemExit(f"В {ICONS_DIR} нет *.svg")

    out_dir.mkdir(parents=True, exist_ok=True)

    total = len(svg_files)
    print(f"Найдено SVG иконок: {total}")
    print(f"Inkscape: {inkscape}")
    print(f"Источник: {icons_dir}")
    print(f"Выходная папка: {out_dir}")
    print(f"Параллельных задач: {args.jobs}")
    print()

    # --- Первый прогон: параллельно ---
    futures = []
    with ThreadPoolExecutor(max_workers=args.jobs) as pool:
        for src in svg_files:
            dst = out_dir / src.name
            futures.append(pool.submit(_process_one, inkscape, src, dst, args.force))

        errors_parallel: list[Tuple[Path, str]] = []
        done_count = 0
        for fut in as_completed(futures):
            src, ok, stderr = fut.result()
            done_count += 1
            prefix = "OK " if ok else "ERR"
            print(f"[{done_count}/{total}] {prefix} {src.name}")
            if not ok:
                errors_parallel.append((src, stderr))

    # --- Второй прогон: последовательный только для упавших файлов ---
    errors_final: list[Tuple[Path, str]] = []
    if errors_parallel:
        print(
            f"\nПовторная попытка для {len(errors_parallel)} файлов "
            "в однопоточном режиме (часто помогает при Gio::DBus::Error)...\n"
        )
        for src, _ in errors_parallel:
            dst = out_dir / src.name
            _, ok, stderr = _process_one(inkscape, src, dst, force=True)
            prefix = "OK " if ok else "ERR"
            print(f"[retry] {prefix} {src.name}")
            if not ok:
                errors_final.append((src, stderr))

    if errors_final:
        print("\nОбнаружены ошибки при обработке следующих файлов (после двух попыток):")
        for src, stderr in errors_final:
            print(f"\n=== {src.name} ===")
            print(stderr)
        return 1

    print("\nГотово: все иконки обработаны и сохранены в icons-expanded/.")
    return 0


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())



