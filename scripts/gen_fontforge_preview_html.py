#!/usr/bin/env python3
"""Генерация HTML-просмотрщика для lucide.fontforge.ttf.

HTML рендерит сетку иконок в том же порядке, в котором
скрипт svg_pack_to_ttf_fontforge.py создаёт глифы:
sorted(tp/lucide/icons/*.svg), кодпоинты от U+E000.
"""

from __future__ import annotations

import html
from pathlib import Path

START_CODEPOINT = 0xE000
BASE_ICONS_DIR = Path("c:/iconflow/tp/lucide/icons")
EXPANDED_ICONS_DIR = BASE_ICONS_DIR.parent / "icons-expanded"
TTF_PATH = Path("c:/iconflow/tp/lucide/lucide-font/lucide.fontforge.ttf")
OUT_HTML = Path("c:/iconflow/tp/lucide/lucide-font/lucide.fontforge-preview.html")


def main() -> None:
    if not TTF_PATH.exists():
        raise SystemExit(f"TTF не найден: {TTF_PATH}")

    # Для визуальной проверки сравниваем TTF с теми же SVG,
    # из которых он собирался: icons-expanded, если есть, иначе icons.
    icons_dir = EXPANDED_ICONS_DIR if EXPANDED_ICONS_DIR.exists() else BASE_ICONS_DIR

    if not icons_dir.exists():
        raise SystemExit(f"Папка с иконками не найдена: {icons_dir}")

    svg_files = sorted([p for p in icons_dir.glob("*.svg") if p.is_file()])
    if not svg_files:
        raise SystemExit(f"В {icons_dir} нет *.svg")

    rows = []
    codepoint = START_CODEPOINT
    for svg in svg_files:
        name = svg.stem
        cp_hex = f"{codepoint:04X}"
        # Путь к исходному SVG относителен HTML-файла
        rel_folder = "icons-expanded" if EXPANDED_ICONS_DIR.exists() else "icons"
        svg_rel = f"../{rel_folder}/{html.escape(svg.name)}"
        rows.append(
            f"<div class='icon-item'>"
            f"  <div class='icon-row'>"
            f"    <div class='icon-glyph'>&#x{cp_hex};</div>"
            f"    <img class='icon-svg' src='{svg_rel}' alt='{html.escape(name)}' />"
            f"  </div>"
            f"  <div class='icon-meta'>"
            f"    <div class='icon-name'>{html.escape(name)}</div>"
            f"    <div class='icon-code'>U+{cp_hex}</div>"
            f"  </div>"
            f"</div>"
        )
        codepoint += 1

    html_doc = f"""<!DOCTYPE html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\" />
  <title>Lucide FontForge Preview</title>
  <style>
    @font-face {{
      font-family: 'LucideFontforge';
      src: url('./lucide.fontforge.ttf') format('truetype');
      font-weight: normal;
      font-style: normal;
    }}
    body {{
      font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      margin: 0;
      padding: 16px;
      background: #0f172a;
      color: #e5e7eb;
    }}
    h1 {{
      margin: 0 0 8px;
      font-size: 20px;
    }}
    p {{
      margin: 4px 0 16px;
      font-size: 13px;
      color: #9ca3af;
    }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
      gap: 12px;
    }}
    .icon-item {{
      background: #020617;
      border-radius: 8px;
      border: 1px solid #1e293b;
      padding: 8px;
      display: flex;
      flex-direction: column;
      gap: 4px;
    }}
    .icon-row {{
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 4px;
    }}
    .icon-glyph {{
      font-family: 'LucideFontforge';
      font-size: 32px;
      line-height: 1;
      width: 40px;
      height: 40px;
      display: flex;
      align-items: center;
      justify-content: center;
    }}
    .icon-svg {{
      width: 32px;
      height: 32px;
      object-fit: contain;
      /* SVG после expand обычно чёрные, поэтому инвертируем,
         чтобы они визуально совпадали с цветом глифа на тёмном фоне. */
      filter: invert(0.9);
      opacity: 0.9;
    }}
    .icon-meta {{
      text-align: center;
      font-size: 11px;
    }}
    .icon-name {{
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 100%;
    }}
    .icon-code {{
      color: #6b7280;
      font-feature-settings: 'tnum' 1, 'lnum' 1;
    }}
  </style>
</head>
<body>
  <h1>Lucide FontForge Preview</h1>
  <p>Шрифт: {html.escape(str(TTF_PATH.name))}, глифы от U+{START_CODEPOINT:04X} по порядку файлов в {html.escape(str(icons_dir))}.</p>
  <div class='grid'>
    {''.join(rows)}
  </div>
</body>
</html>
"""

    OUT_HTML.write_text(html_doc, encoding="utf-8")
    print(f"HTML-превью создано: {OUT_HTML}")


if __name__ == "__main__":
    main()




