mod api;
mod error;
mod types;

pub use api::{fonts, list, try_icon};
pub use error::IconError;
pub use types::{FontAsset, IconRef, Size, Style};

// Re-exported for `crate::core::{IconEntry, VariantKey}` paths in generated code
// (those paths are cfg-gated on pack features, so the imports look unused otherwise).
#[allow(unused_imports)]
pub(crate) use types::{IconEntry, VariantKey};
