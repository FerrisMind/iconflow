# iconflow

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/iconflow/main/assets/icons/iconflow-color.png" alt="iconflow" width="200" />
</p>

<p align="center">
  <a href="https://crates.io/crates/iconflow">
    <img src="https://img.shields.io/crates/v/iconflow.svg" alt="crates.io version" />
  </a>
  <a href="https://github.com/FerrisMind/iconflow/blob/main/LICENSE">
    <img src="https://img.shields.io/crates/l/iconflow.svg" alt="license" />
  </a>
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
  <img src="https://raw.githubusercontent.com/FerrisMind/iconflow/main/examples/v1.0/egui_demo/egui_demo.png" alt="egui_demo" width="900" />
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
        definitions.font_data.insert(
            font.family.to_string(),
            Arc::new(FontData::from_static(font.bytes)),
        );
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
  <img src="https://raw.githubusercontent.com/FerrisMind/iconflow/main/examples/v1.0/iced_demo/iced_demo.png" alt="iced_demo" width="900" />
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

See [docs/faq.md](https://raw.githubusercontent.com/FerrisMind/iconflow/main/docs/faq.md).

## Development

- `cargo xtask gen` regenerates `src/generated/**` from `assets/maps/*.json`.
- `cargo xtask gen --check` verifies generated output without writing files.

## Icon Fonts and Licenses

iconflow includes icon fonts from 14 open-source icon packs. All fonts are distributed under permissive licenses (MIT, Apache-2.0, or ISC).

### Included Icon Packs

| Icon Pack | License | Source |
|-----------|---------|--------|
| [Bootstrap Icons](https://github.com/twbs/icons) | MIT | [twbs/icons](https://github.com/twbs/icons) |
| [Carbon Icons](https://github.com/carbon-design-system/carbon-icons) | Apache-2.0 | [carbon-design-system/carbon-icons](https://github.com/carbon-design-system/carbon-icons) |
| [Devicon](https://github.com/devicons/devicon) | MIT | [devicons/devicon](https://github.com/devicons/devicon) |
| [Feather Icons](https://github.com/feathericons/feather) | MIT | [feathericons/feather](https://github.com/feathericons/feather) |
| [Fluent UI System Icons](https://github.com/microsoft/fluentui-system-icons) | MIT | [microsoft/fluentui-system-icons](https://github.com/microsoft/fluentui-system-icons) |
| [Heroicons](https://github.com/tailwindlabs/heroicons) | MIT | [tailwindlabs/heroicons](https://github.com/tailwindlabs/heroicons) |
| [Iconoir](https://github.com/iconoir-icons/iconoir) | MIT | [iconoir-icons/iconoir](https://github.com/iconoir-icons/iconoir) |
| [Ionicons](https://github.com/ionic-team/ionicons) | MIT | [ionic-team/ionicons](https://github.com/ionic-team/ionicons) |
| [Lobe Icons](https://github.com/lobehub/lobe-icons) | MIT | [lobehub/lobe-icons](https://github.com/lobehub/lobe-icons) |
| [Lucide](https://github.com/lucide-icons/lucide) | ISC | [lucide-icons/lucide](https://github.com/lucide-icons/lucide) |
| [Octicons](https://github.com/primer/octicons) | MIT | [primer/octicons](https://github.com/primer/octicons) |
| [Phosphor Icons](https://github.com/phosphor-icons/web) | MIT | [phosphor-icons/web](https://github.com/phosphor-icons/web) |
| [Remix Icon](https://github.com/Remix-Design/remixicon) | Apache-2.0 | [Remix-Design/remixicon](https://github.com/Remix-Design/remixicon) |
| [Tabler Icons](https://github.com/tabler/tabler-icons) | MIT | [tabler/tabler-icons](https://github.com/tabler/tabler-icons) |

**Total:** 34 TTF font files across 14 icon packs.

### Acknowledgments

iconflow is built on top of these excellent open-source icon libraries. We are grateful to all the contributors and maintainers of these projects:

- **Bootstrap Icons** by Bootstrap team
- **Carbon Icons** by IBM Carbon Design System
- **Devicon** by Devicon and contributors
- **Feather Icons** by Cole Bemis and contributors
- **Fluent UI System Icons** by Microsoft
- **Heroicons** by Tailwind Labs
- **Iconoir** by Luca Burgio and contributors
- **Ionicons** by Ionic team
- **Lobe Icons** by LobeHub
- **Lucide** by Lucide contributors
- **Octicons** by GitHub (Primer)
- **Phosphor Icons** by Phosphor Icons team
- **Remix Icon** by Remix Design
- **Tabler Icons** by codecalm and contributors

All icon fonts are converted from their original SVG sources and embedded as TTF files in this project. The fonts are included in the crate binary when corresponding feature flags are enabled.

## License

This project is licensed under MIT License.

Third-party dependencies and fonts have their own licenses. See:
- [THIRD_PARTY_LICENSES_CRATES.html](https://raw.githubusercontent.com/FerrisMind/iconflow/main/THIRD_PARTY_LICENSES_CRATES.html) for Rust crate licenses
- [THIRD_PARTY_LICENSES_FONTS.md](https://raw.githubusercontent.com/FerrisMind/iconflow/main/THIRD_PARTY_LICENSES_FONTS.md) for font licenses
