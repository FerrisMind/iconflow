# Quickstart

This guide shows the fastest way to integrate iconflow into a Rust GUI app and
use the core API.

## Install

Enable at least one pack feature so font assets and icon data are included.

```toml
[dependencies]
iconflow = { version = "0.1", features = ["all-packs"] }
```

## Core API at a glance

- `fonts()` returns `FontAsset` entries for enabled packs.
- `try_icon(pack, name, style, size)` returns an `IconRef` or `IconError`.
- `list(pack)` returns all icon names for a pack.

```rust
use iconflow::{fonts, list, try_icon, Pack, Size, Style};

let _fonts = fonts();
let names = list(Pack::Bootstrap);
let icon = try_icon(Pack::Bootstrap, "alarm", Style::Regular, Size::Regular)?;
```

## egui integration (minimal)

Register fonts and render the icon glyph with `FontFamily::Name`.

```rust
use egui::{FontData, FontDefinitions, FontFamily, FontId, RichText};
use iconflow::{fonts, try_icon, Pack, Size, Style};

fn install_icon_fonts(ctx: &egui::Context) {
    let mut definitions = FontDefinitions::default();
    for font in fonts() {
        definitions
            .font_data
            .insert(font.family.to_string(), FontData::from_static(font.bytes));
        let family = definitions
            .families
            .entry(FontFamily::Name(font.family.into()))
            .or_default();
        family.insert(0, font.family.to_string());
    }
    ctx.set_fonts(definitions);
}

fn icon_label(ui: &mut egui::Ui) {
    let icon = try_icon(Pack::Bootstrap, "alarm", Style::Regular, Size::Regular)
        .expect("icon missing");
    let glyph = char::from_u32(icon.codepoint).unwrap_or('?');
    let font_id = FontId::new(32.0, FontFamily::Name(icon.family.into()));
    ui.label(RichText::new(glyph.to_string()).font(font_id));
}
```

## iced integration (minimal)

Load fonts through `Task`, then render a glyph with `Font::with_name`.

```rust
use iced::{Task, font};
use iced::widget::text;
use iconflow::{fonts, try_icon, Pack, Size, Style};

fn load_all_fonts() -> Task<()> {
    Task::batch(fonts().iter().map(|font| font::load(font.bytes).map(|_| ())))
}

fn icon_text() -> iced::widget::Text<'static> {
    let icon = try_icon(Pack::Bootstrap, "alarm", Style::Regular, Size::Regular)
        .expect("icon missing");
    let glyph = char::from_u32(icon.codepoint).unwrap_or('?');
    text(glyph.to_string())
        .size(48)
        .font(iced::font::Font::with_name(icon.family))
}
```

## Run the examples

```bash
cargo run --example egui_demo --features all-packs
cargo run --example iced_demo --features all-packs
```
