//! Unified icon packs for Rust GUI apps (egui, iced, and similar).
//!
//! Enable the pack features you need (for example `pack-bootstrap`), register
//! fonts from [`fonts`], then resolve glyphs with [`try_icon`].
//!
//! # Examples
//!
//! Default features (no packs enabled) still exercise the public types:
//!
//! ```
//! use iconflow::{IconError, Size, Style};
//!
//! assert_eq!(Size::default(), Size::Regular);
//! assert_eq!(Style::default(), Style::Regular);
//! assert_eq!(Size::Custom(24).to_string(), "custom(24)");
//! assert_eq!(Style::Filled.to_string(), "filled");
//!
//! let err = IconError::PackDisabled { pack: "bootstrap" };
//! assert_eq!(err.to_string(), "pack `bootstrap` is disabled");
//! ```
//!
//! With a pack feature, resolve a glyph by name:
//!
//! ```
//! # #[cfg(feature = "pack-bootstrap")]
//! # fn example() -> Result<(), iconflow::IconError> {
//! use iconflow::{try_icon, Pack, Size, Style};
//!
//! let icon = try_icon(Pack::Bootstrap, "alarm", Style::Regular, Size::Regular)?;
//! assert_eq!(icon.family, "Bootstrap Regular");
//! # let _ = icon;
//! # Ok(())
//! # }
//! # #[cfg(feature = "pack-bootstrap")]
//! # example().unwrap();
//! ```

#![warn(missing_docs)]

pub(crate) mod core;
pub(crate) mod generated;

pub use crate::core::{FontAsset, IconError, IconRef, Size, Style, fonts, list, try_icon};
pub use crate::generated::Pack;

#[cfg(test)]
mod send_sync_tests {
    use super::{FontAsset, IconError, IconRef, Pack, Size, Style};

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_sync() {
        assert_send_sync::<Size>();
        assert_send_sync::<Style>();
        assert_send_sync::<FontAsset>();
        assert_send_sync::<IconRef>();
        assert_send_sync::<Pack>();
        assert_send_sync::<IconError>();
    }
}
