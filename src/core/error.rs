use std::borrow::Cow;
use std::fmt;

use crate::core::{Size, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IconError {
    PackDisabled {
        pack: &'static str,
    },
    IconNotFound {
        pack: &'static str,
        name: Cow<'static, str>,
    },
    VariantUnavailable {
        pack: &'static str,
        name: Cow<'static, str>,
        requested: (Style, Size),
        available: &'static [(Style, Size)],
    },
}

impl fmt::Display for IconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IconError::PackDisabled { pack } => {
                write!(f, "pack `{pack}` is disabled")
            }
            IconError::IconNotFound { pack, name } => {
                write!(f, "icon `{name}` not found in pack `{pack}`")
            }
            IconError::VariantUnavailable {
                pack,
                name,
                requested: (style, size),
                ..
            } => {
                write!(
                    f,
                    "variant {style:?}/{size:?} unavailable for icon `{name}` in pack `{pack}`"
                )
            }
        }
    }
}

impl std::error::Error for IconError {}
