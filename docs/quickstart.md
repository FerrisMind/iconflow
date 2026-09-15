# Quickstart

This guide shows the fastest way to integrate iconflow into a Rust GUI app and
use the core API.

## Install

Enable at least one pack feature so font assets and icon data are included.

```toml
[dependencies]
iconflow = { version = "2.1", features = ["all-packs"] }
```

For egui, also depend on `eframe` (iconflow itself has no GUI dependencies):

```toml
eframe = "0.33"
```

## Core API at a glance

- `fonts()` returns `FontAsset` entries for enabled packs.
- `try_icon(pack, name, style, size)` returns an `IconRef` or `IconError`.
- `list(pack)` returns all icon names for a pack.
- `resolve_all(pack, style, size)` returns a full-pack `Vec` of `Result<IconRef, IconError>` in `list` order.

```rust
use iconflow::{fonts, list, try_icon, Pack, Size, Style};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _fonts = fonts();
    let names = list(Pack::Bootstrap);
    let icon = try_icon(Pack::Bootstrap, "alarm", Style::Regular, Size::Regular)?;
    let _ = (names, icon);
    Ok(())
}
```

For icon pickers, call `resolve_all` once as the cold path (linear table walk; same order as
`list`) instead of `n × try_icon`. Keep that `Vec` and index it on warm frames. See
[faq.md](faq.md#icon-pickers-cold--warm-frames).

### Packs without `(Style::Regular, Size::Regular)`

Bootstrap (and several other packs) accept Regular/Regular, but **heroicons** and
**remixicon** do not — they expose `Filled` / `Outline` at `Size::Regular` instead.
Calling `try_icon(..., Style::Regular, Size::Regular)` on those packs yields
`IconError::VariantUnavailable`; use the `available` field on the error to choose a
valid pair. See [faq.md](faq.md#default-styleregular-sizeregular-is-not-universal-r3-n-07).

## egui integration (minimal)

Register fonts and render the icon glyph with `FontFamily::Name`. Use `eframe::egui`
(matching the demo) and wrap font data in `Arc` — egui 0.33 expects `Arc<FontData>`.

```rust
use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, RichText};
use iconflow::{fonts, try_icon, Pack, Size, Style};
use std::sync::Arc;

fn install_icon_fonts(ctx: &egui::Context) {
    let mut definitions = FontDefinitions::default();
    for font in fonts() {
        definitions.font_data.insert(
            font.family.to_string(),
            Arc::new(FontData::from_static(font.bytes)),
        );
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
