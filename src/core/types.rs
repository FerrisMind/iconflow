/// Canonical size variants for icon packs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Size {
    /// Smallest available size (pack-specific).
    Tiny,
    /// Small size variant (pack-specific).
    Mini,
    /// Default size variant.
    Regular,
    /// Larger size variant when a pack provides it.
    Large,
    /// Custom size requested by the consumer.
    Custom(u16),
}

/// Canonical style variants for icon packs.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Style {
    Regular,
    Filled,
    Outline,
    Light,
    Thin,
    Bold,
    Duotone,
    Glyph,
    Sharp,
    Rounded,
}

/// Font bytes and family name for a specific variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FontAsset {
    /// Font family name stored inside the TTF.
    pub family: &'static str,
    /// Raw font bytes.
    pub bytes: &'static [u8],
}

/// Reference to a concrete glyph inside a font.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct IconRef {
    /// Font family name stored inside the TTF.
    pub family: &'static str,
    /// Unicode codepoint of the glyph.
    pub codepoint: u32,
}

/// Variant key used to index font assets and codepoints.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct VariantKey {
    pub style: Style,
    pub size: Size,
}

#[cfg(test)]
mod tests {
    use super::{Size, Style, VariantKey};

    #[test]
    fn variant_key_compares_by_fields() {
        let left = VariantKey {
            style: Style::Regular,
            size: Size::Regular,
        };
        let right = VariantKey {
            style: Style::Regular,
            size: Size::Regular,
        };
        assert_eq!(left, right);
    }
}
