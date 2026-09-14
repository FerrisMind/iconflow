use std::cmp::Ordering;
use std::fmt;

/// Canonical size variants for icon packs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Size {
    /// Smallest available size (pack-specific).
    Tiny,
    /// Small size variant (pack-specific).
    Mini,
    /// Default size variant.
    #[default]
    Regular,
    /// Larger size variant when a pack provides it.
    Large,
    /// Exact size key from a pack map (pixel or design size as `u16`).
    ///
    /// Matching is **exact**: `Custom(n)` only resolves when the pack table
    /// lists that same `n` for the icon. Passing an arbitrary `u16` does **not**
    /// synthesize, scale, or invent a font; unavailable keys yield
    /// [`crate::IconError::VariantUnavailable`] (or a miss on the name path).
    Custom(u16),
}

impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Size::Tiny => f.write_str("tiny"),
            Size::Mini => f.write_str("mini"),
            Size::Regular => f.write_str("regular"),
            Size::Large => f.write_str("large"),
            Size::Custom(n) => write!(f, "custom({n})"),
        }
    }
}

/// Canonical style variants for icon packs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Style {
    /// Default / regular weight style.
    #[default]
    Regular,
    /// Filled style.
    Filled,
    /// Outline style.
    Outline,
    /// Light weight.
    Light,
    /// Thin weight.
    Thin,
    /// Bold weight.
    Bold,
    /// Duotone style.
    Duotone,
    /// Glyph / symbol style.
    Glyph,
    /// Sharp corners style.
    Sharp,
    /// Rounded corners style.
    Rounded,
}

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Style::Regular => f.write_str("regular"),
            Style::Filled => f.write_str("filled"),
            Style::Outline => f.write_str("outline"),
            Style::Light => f.write_str("light"),
            Style::Thin => f.write_str("thin"),
            Style::Bold => f.write_str("bold"),
            Style::Duotone => f.write_str("duotone"),
            Style::Glyph => f.write_str("glyph"),
            Style::Sharp => f.write_str("sharp"),
            Style::Rounded => f.write_str("rounded"),
        }
    }
}

/// Font bytes and family name for a specific variant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FontAsset {
    /// Font family name stored inside the TTF.
    pub family: &'static str,
    /// Raw font bytes.
    pub bytes: &'static [u8],
}

impl PartialOrd for FontAsset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FontAsset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.family
            .cmp(other.family)
            .then_with(|| self.bytes.cmp(other.bytes))
    }
}

impl fmt::Display for FontAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.family)
    }
}

/// Reference to a concrete glyph inside a font.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct IconRef {
    /// Font family name stored inside the TTF.
    pub family: &'static str,
    /// Unicode codepoint of the glyph.
    pub codepoint: u32,
}

impl PartialOrd for IconRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IconRef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.family
            .cmp(other.family)
            .then_with(|| self.codepoint.cmp(&other.codepoint))
    }
}

impl fmt::Display for IconRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:U+{:X}", self.family, self.codepoint)
    }
}

/// Variant key used to index font assets and codepoints.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[allow(dead_code)] // constructed by generated pack tables when pack features are enabled
pub(crate) struct VariantKey {
    pub style: Style,
    pub size: Size,
}

impl fmt::Display for VariantKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.style, self.size)
    }
}

/// Crate-private descriptor for one icon in a generated pack table.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)] // referenced by generated pack tables when pack features are enabled
pub(crate) struct IconEntry {
    pub name: &'static str,
    pub variants: &'static [(VariantKey, u32)],
    pub available: &'static [(Style, Size)],
}

#[cfg(test)]
mod tests {
    use super::{FontAsset, IconRef, Size, Style, VariantKey};

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

    #[test]
    fn defaults_match_locked_matrix() {
        assert_eq!(Size::default(), Size::Regular);
        assert_eq!(Style::default(), Style::Regular);
        assert_eq!(
            VariantKey::default(),
            VariantKey {
                style: Style::Regular,
                size: Size::Regular,
            }
        );
    }

    #[test]
    fn display_is_lowercase() {
        assert_eq!(Size::Regular.to_string(), "regular");
        assert_eq!(Size::Custom(24).to_string(), "custom(24)");
        assert_eq!(Style::Filled.to_string(), "filled");
        assert_eq!(
            VariantKey {
                style: Style::Outline,
                size: Size::Mini,
            }
            .to_string(),
            "outline/mini"
        );
    }

    #[test]
    fn font_asset_and_icon_ref_ord_by_family() {
        let a = FontAsset {
            family: "A",
            bytes: b"1",
        };
        let b = FontAsset {
            family: "B",
            bytes: b"0",
        };
        assert!(a < b);
        let ia = IconRef {
            family: "A",
            codepoint: 2,
        };
        let ib = IconRef {
            family: "A",
            codepoint: 1,
        };
        assert!(ib < ia);
    }
}
