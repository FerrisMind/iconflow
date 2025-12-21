# iconflow

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/iconflow/main/assets/icons/iconflow-color.png" alt="iconflow" width="200" />
</p>

## Install

Enable at least one pack feature so the fonts and icon data are included.

```toml
[dependencies]
iconflow = { version = "1.0", features = ["all-packs"] }
```

## Quickstart guide

See `docs/quickstart.md` for a fast end-to-end setup guide and API overview.

## Core API

- `fonts()` returns the enabled font assets for registered packs.
- `try_icon(pack, name, style, size)` resolves an icon reference or returns `IconError`.
- `list(pack)` returns the icon names for a pack.

## egui quickstart

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/iconflow/main/examples/egui_demo/egui_demo.png" alt="egui_demo" width="900" />
</p>

Register every `FontAsset` in `egui::FontDefinitions`, then render `IconRef.codepoint` with
`FontFamily::Name(icon.family)`:

```rust
use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, RichText};
use iconflow::{fonts, try_icon, Pack, Size, Style};
use std::sync::Arc;

fn install_icon_fonts(ctx: &egui::Context) {
    let mut definitions = FontDefinitions::default();
    let fallback_fonts: Vec<String> = definitions.font_data.keys().cloned().collect();

    for font in fonts() {
        definitions
            .font_data
            .insert(font.family.to_string(), Arc::new(FontData::from_static(font.bytes)));
        let family = definitions
            .families
            .entry(FontFamily::Name(font.family.into()))
            .or_default();
        family.insert(0, font.family.to_string());
        for fallback in &fallback_fonts {
            if fallback != font.family {
                family.push(fallback.clone());
            }
        }
    }

    ctx.set_fonts(definitions);
}

fn icon_label(ui: &mut egui::Ui) {
    let samples = [
        (Pack::Bootstrap, "alarm", Style::Regular, Size::Regular),
        (Pack::Bootstrap, "alarm", Style::Filled, Size::Regular),
        (Pack::Heroicons, "academic-cap", Style::Filled, Size::Regular),
    ];

    for (pack, name, style, size) in samples {
        let icon = try_icon(pack, name, style, size).expect("icon missing");
        let glyph = char::from_u32(icon.codepoint).unwrap_or('?');
        let font_id = FontId::new(32.0, FontFamily::Name(icon.family.into()));

        ui.label(RichText::new(glyph.to_string()).font(font_id));
    }
}
```

Runnable example: `cargo run --example egui_demo --features all-packs`

## iced 0.14 quickstart

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/iconflow/main/examples/iced_demo/iced_demo.png" alt="iced_demo" width="900" />
</p>

In iced 0.14, fonts are loaded asynchronously via `Task`. Load the bytes from `fonts()` and
render a glyph with `Font::with_name` once loading completes.

```rust
use iced::{Task, font};
use iced::widget::text;
use iconflow::{fonts, try_icon, Pack, Size, Style};

#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<(), font::Error>),
}

fn load_all_fonts() -> Task<Message> {
    Task::batch(fonts().iter().map(|font| {
        font::load(font.bytes).map(Message::FontLoaded)
    }))
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

Runnable example: `cargo run --example iced_demo --features all-packs`

Example READMEs: `examples/egui_demo/README.md`, `examples/iced_demo/README.md`.

## FAQ

See `docs/faq.md`.

## Development

- `cargo xtask gen` regenerates `src/generated/**` from `assets/maps/*.json`.
- `cargo xtask gen --check` verifies generated output without writing files.

## License

MIT