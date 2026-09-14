//! Unified icon packs for Rust GUI apps (egui, iced, and similar).
//!
//! Enable the pack features you need (for example `pack-bootstrap`), register
//! fonts from [`fonts`], then resolve glyphs with [`try_icon`].
//!
//! # Example
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

#[doc(hidden)]
pub mod core;
pub(crate) mod generated;

pub use crate::core::{FontAsset, IconError, IconRef, Size, Style, fonts, list, try_icon};
pub use crate::generated::Pack;
