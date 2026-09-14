use std::borrow::Cow;
use std::fmt;

use crate::core::{Size, Style};

/// Errors returned by [`crate::try_icon`] and related lookup helpers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum IconError {
    /// The requested pack feature is not enabled (or no packs are enabled).
    PackDisabled {
        /// Pack identifier (stable string key).
        pack: &'static str,
    },
    /// No icon with this name exists in the pack.
    IconNotFound {
        /// Pack identifier (stable string key).
        pack: &'static str,
        /// Looked-up icon name.
        name: Cow<'static, str>,
    },
    /// The icon exists, but the requested `(style, size)` is not available.
    VariantUnavailable {
        /// Pack identifier (stable string key).
        pack: &'static str,
        /// Icon name from the pack table.
        name: Cow<'static, str>,
        /// Requested style and size.
        requested: (Style, Size),
        /// Variants present for this icon.
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
                    "variant {style}/{size} unavailable for icon `{name}` in pack `{pack}`"
                )
            }
        }
    }
}

impl std::error::Error for IconError {}
