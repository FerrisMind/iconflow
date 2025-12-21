use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, RichText};
use iconflow::{IconError, Pack, Size, Style, fonts, list, try_icon};
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "iconflow egui demo",
        options,
        Box::new(|cc| Ok(Box::new(IconDemo::new(cc)))),
    )
}

struct IconDemo;

impl IconDemo {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
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

        cc.egui_ctx.set_fonts(definitions);
        Self
    }
}

impl eframe::App for IconDemo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Заголовок вверху слева
            ui.heading("iconflow egui demo");
            ui.add_space(20.0);

            let packs = [
                Pack::Bootstrap,
                Pack::Carbon,
                Pack::Devicon,
                Pack::Feather,
                Pack::Fluentui,
                Pack::Heroicons,
                Pack::Iconoir,
                Pack::Ionicons,
                Pack::Lobe,
                Pack::Lucide,
                Pack::Octicons,
                Pack::Phosphor,
                Pack::Remixicon,
                Pack::Tabler,
            ];
            let mut items = Vec::new();

            for pack in packs {
                if let Some(name) = list(pack).first().copied() {
                    items.push(resolve_icon(pack, name));
                } else {
                    items.push(Err(format!("{pack:?}: no icons")));
                }
            }

            const COLUMNS: usize = 4;
            let available_rect = ui.available_rect_before_wrap();
            let available_width = available_rect.width();
            let grid_width = (COLUMNS as f32 * 150.0).min(available_width * 0.8);

            // Вычисляем отступы для центрирования
            let horizontal_padding = (available_width - grid_width) / 2.0;

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                // Добавляем отступ слева для центрирования
                ui.add_space(horizontal_padding);

                // Ограничиваем ширину Grid через max_rect
                let grid_rect = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::Vec2::new(grid_width, f32::INFINITY),
                );

                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(grid_rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                    |ui| {
                        egui::Grid::new("icon_grid")
                            .num_columns(COLUMNS)
                            .spacing([24.0, 16.0])
                            .show(ui, |ui| {
                                for (index, item) in items.into_iter().enumerate() {
                                    match item {
                                        Ok(sample) => {
                                            let glyph = char::from_u32(sample.icon.codepoint)
                                                .unwrap_or('?');
                                            let font_id = FontId::new(
                                                32.0,
                                                FontFamily::Name(sample.icon.family.into()),
                                            );
                                            ui.vertical_centered(|ui| {
                                                ui.label(
                                                    RichText::new(glyph.to_string()).font(font_id),
                                                );
                                                ui.label(sample.label);
                                            });
                                        }
                                        Err(message) => {
                                            ui.label(message);
                                        }
                                    }

                                    if (index + 1) % COLUMNS == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    },
                );
            });

            // Текст о количестве шрифтов внизу
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                let fonts_count = fonts().len();
                ui.label(format!("Fonts loaded: {fonts_count}/{fonts_count}"));
            });
        });
    }
}

struct Sample {
    icon: iconflow::IconRef,
    label: String,
}

fn resolve_icon(pack: Pack, name: &'static str) -> Result<Sample, String> {
    let mut style = Style::Regular;
    let mut size = Size::Regular;
    let icon = match try_icon(pack, name, style, size) {
        Ok(icon) => icon,
        Err(IconError::VariantUnavailable { available, .. }) => {
            if let Some((next_style, next_size)) = available.first().copied() {
                style = next_style;
                size = next_size;
                try_icon(pack, name, style, size).map_err(|err| format!("{err:?}"))?
            } else {
                return Err(format!("{pack:?}: no available variants"));
            }
        }
        Err(err) => {
            return Err(format!("{err:?}"));
        }
    };

    Ok(Sample {
        icon,
        label: format!("{pack:?} / {name} / {style:?} / {size:?}"),
    })
}
