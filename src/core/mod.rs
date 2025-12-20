mod api;
mod error;
mod types;

pub use api::{fonts, list, try_icon};
pub use error::IconError;
pub use types::{FontAsset, IconRef, Size, Style, VariantKey};
