#[doc(hidden)]
pub mod core;
pub(crate) mod generated;

pub use crate::core::{FontAsset, IconError, IconRef, Size, Style, fonts, list, try_icon};
pub use crate::generated::Pack;
