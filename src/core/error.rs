use std::borrow::Cow;
use std::fmt;

use crate::core::{Size, Style};

/// Errors returned by [`crate::try_icon`] and related lookup helpers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum IconError {
    /// Pack feature absent / lookup path reports a disabled pack.
    ///
    /// **Normal feature use does not return this.** Missing `pack-*` features make
    /// [`crate::Pack`] an empty enum (or omit the variant), so [`crate::list`] /
    /// [`crate::try_icon`] / [`crate::resolve_all`] fail at **compile time**, not with
    /// `PackDisabled`. The only in-tree constructor is the zero-feature stub in
    /// `src/generated` (`pack: "none"`), which is unreachable via a constructible
    /// `Pack` value. Kept for a stable, `#[non_exhaustive]` error surface and
    /// match exhaustiveness — not as the “forgot to enable a pack” signal.
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
