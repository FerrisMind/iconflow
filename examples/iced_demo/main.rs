use iced::widget::{column, container, row, text};
use iced::{Element, Length, Task, Theme, font};
use iconflow::{IconError, Pack, Size, Style, list, try_icon};

fn main() -> iced::Result {
    iced::application(IconDemo::new, IconDemo::update, IconDemo::view)
        .title("iconflow iced demo")
        .theme(IconDemo::theme)
        .run()
}

struct IconDemo {
    fonts_total: usize,
    fonts_loaded: usize,
    font_error: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    FontLoaded(Result<(), font::Error>),
}

impl IconDemo {
    fn new() -> (Self, Task<Message>) {
        let fonts_total = iconflow::fonts().len();
        let tasks = iconflow::fonts()
            .iter()
            .map(|font| font::load(font.bytes).map(Message::FontLoaded));

        (
            Self {
                fonts_total,
                fonts_loaded: 0,
                font_error: None,
            },
            Task::batch(tasks),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FontLoaded(result) => {
                self.fonts_loaded += 1;
                if let Err(err) = result && self.font_error.is_none() {
                    self.font_error = Some(format!("{err:?}"));
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let status = if let Some(err) = &self.font_error {
            format!("Font load error: {err}")
        } else {
            format!("Fonts loaded: {}/{}", self.fonts_loaded, self.fonts_total)
        };

        if self.fonts_loaded < self.fonts_total {
            return column![text("Loading icon fonts..."), text(status)]
                .spacing(8)
                .padding(20)
                .into();
        }

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
        let mut cells = Vec::new();

        for pack in packs {
            if let Some(name) = list(pack).first().copied() {
                cells.push(resolve_icon(pack, name));
            } else {
                cells.push(Err(format!("{pack:?}: no icons")));
            }
        }

        const COLUMNS: usize = 4;
        let mut grid = column![text("iconflow iced demo").size(24)].spacing(16);
        let mut current_row = row![].spacing(24);
        let mut row_len = 0usize;

        for (index, cell) in cells.into_iter().enumerate() {
            let item: Element<'_, Message> = match cell {
                Ok(sample) => {
                    let glyph = char::from_u32(sample.icon.codepoint).unwrap_or('?');
                    let icon_text = text::<Theme, iced::Renderer>(glyph.to_string())
                        .size(48)
                        .font(iced::font::Font::with_name(sample.icon.family));
                    let label = text::<Theme, iced::Renderer>(sample.label);
                    column![icon_text, label]
                        .spacing(6)
                        .align_x(iced::alignment::Horizontal::Center)
                        .into()
                }
                Err(message) => text::<Theme, iced::Renderer>(message).size(14).into(),
            };

            current_row = current_row.push(item);
            row_len += 1;

            if (index + 1) % COLUMNS == 0 {
                grid = grid.push(current_row);
                current_row = row![].spacing(24);
                row_len = 0;
            }
        }

        if row_len > 0 {
            grid = grid.push(current_row);
        }

        grid = grid.push(text(status));

        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
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
