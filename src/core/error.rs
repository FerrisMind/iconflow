use crate::core::{Size, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconError {
    PackDisabled {
        pack: &'static str,
    },
    IconNotFound {
        pack: &'static str,
        name: String,
    },
    VariantUnavailable {
        pack: &'static str,
        name: String,
        requested: (Style, Size),
        available: &'static [(Style, Size)],
    },
}
