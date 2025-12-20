#!/usr/bin/env python3
"""
Скрипт для проверки наличия TTF файлов шрифтов в каждой папке иконпака.

Проверяет наличие файлов формата .ttf (TrueType Font).
"""

import os
import shutil
from pathlib import Path
from typing import List, Dict, Tuple


# Расширение TTF файлов
TTF_EXTENSION = '.ttf'
ASSETS_FONTS_DIR = Path('assets') / 'fonts'
LEGACY_FONTS_DIR = Path('fonts')


def find_ttf_files(directory: Path) -> List[Path]:
    """
    Рекурсивно находит все TTF файлы в указанной директории.
    
    Args:
        directory: Путь к директории для поиска
        
    Returns:
        Список путей к найденным TTF файлам
    """
    ttf_files = []
    
    for root, dirs, files in os.walk(directory):
        for file in files:
            file_path = Path(root) / file
            if file_path.suffix.lower() == TTF_EXTENSION:
                ttf_files.append(file_path)
    
    return ttf_files


def check_iconpack_fonts(iconpack_dir: Path) -> Dict[str, any]:
    """
    Проверяет наличие TTF файлов шрифтов в папке иконпака.
    
    Args:
        iconpack_dir: Путь к папке иконпака
        
    Returns:
        Словарь с результатами проверки:
        - has_ttf: bool - есть ли TTF файлы
        - ttf_files: List[Path] - список найденных TTF файлов
        - ttf_count: int - количество TTF файлов
    """
    ttf_files = find_ttf_files(iconpack_dir)
    
    return {
        'has_ttf': len(ttf_files) > 0,
        'ttf_files': ttf_files,
        'ttf_count': len(ttf_files)
    }


def resolve_fonts_dir() -> Tuple[Path, bool]:
    """
    Определяет целевую директорию для шрифтов с поддержкой legacy-раскладки.

    Returns:
        (fonts_dir, is_legacy_layout)
    """
    if ASSETS_FONTS_DIR.exists():
        return ASSETS_FONTS_DIR, False
    if LEGACY_FONTS_DIR.exists():
        return LEGACY_FONTS_DIR, True
    return ASSETS_FONTS_DIR, False


def copy_ttf_to_fonts(
    ttf_file: Path,
    fonts_dir: Path,
    iconpack_name: str,
    is_legacy_layout: bool,
) -> Path:
    """
    Копирует TTF файл в папку fonts с уникальным именем.
    
    Args:
        ttf_file: Путь к исходному TTF файлу
        fonts_dir: Путь к папке fonts
        iconpack_name: Имя иконпака для префикса
        is_legacy_layout: True, если используется устаревшая плоская структура
        
    Returns:
        Путь к скопированному файлу
    """
    # Создаём уникальное имя: iconpack_name-original_name.ttf
    original_name = ttf_file.stem
    new_name = f"{iconpack_name}-{original_name}.ttf"
    if is_legacy_layout:
        dest_path = fonts_dir / new_name
    else:
        pack_dir = fonts_dir / iconpack_name
        pack_dir.mkdir(parents=True, exist_ok=True)
        dest_path = pack_dir / new_name
    
    # Копируем файл
    shutil.copy2(ttf_file, dest_path)
    return dest_path


def main():
    """Основная функция скрипта."""
    # Путь к папке с иконпаками
    tp_dir = Path('tp')
    fonts_dir, is_legacy_layout = resolve_fonts_dir()
    
    if not tp_dir.exists():
        print(f"Ошибка: папка '{tp_dir}' не найдена")
        return
    
    # Создаём папку fonts, если её нет
    fonts_dir.mkdir(parents=True, exist_ok=True)
    
    print("Проверка наличия TTF файлов шрифтов в иконпаках\n")
    print("=" * 80)
    
    results = {}
    copied_files = []
    
    # Проходим по всем папкам в tp/
    for iconpack_path in sorted(tp_dir.iterdir()):
        if not iconpack_path.is_dir():
            continue
        
        iconpack_name = iconpack_path.name
        result = check_iconpack_fonts(iconpack_path)
        results[iconpack_name] = result
        
        # Выводим результат
        status = "[+] ЕСТЬ" if result['has_ttf'] else "[-] НЕТ"
        print(f"\n{iconpack_name}: {status}")
        
        if result['has_ttf']:
            print(f"  Найдено TTF файлов: {result['ttf_count']}")
            
            # Показываем все пути к TTF файлам и копируем их
            print(f"  Файлы:")
            for ttf_file in result['ttf_files']:
                rel_path = ttf_file.relative_to(tp_dir)
                print(f"    {rel_path}")
                
                # Копируем файл в папку fonts
                copied_path = copy_ttf_to_fonts(
                    ttf_file,
                    fonts_dir,
                    iconpack_name,
                    is_legacy_layout,
                )
                copied_files.append(copied_path)
                print(f"      -> скопирован в: {copied_path}")
    
    # Итоговая статистика
    print("\n" + "=" * 80)
    print("\nИтоговая статистика:")
    total_iconpacks = len(results)
    iconpacks_with_ttf = sum(1 for r in results.values() if r['has_ttf'])
    iconpacks_without_ttf = total_iconpacks - iconpacks_with_ttf
    
    print(f"  Всего иконпаков: {total_iconpacks}")
    print(f"  С TTF файлами: {iconpacks_with_ttf}")
    print(f"  Без TTF файлов: {iconpacks_without_ttf}")
    print(f"  Скопировано TTF файлов: {len(copied_files)}")
    
    if iconpacks_without_ttf > 0:
        print(f"\n  Иконпаки без TTF файлов:")
        for name, result in results.items():
            if not result['has_ttf']:
                print(f"    - {name}")


if __name__ == '__main__':
    main()
