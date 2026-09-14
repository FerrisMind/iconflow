use crate::core::{FontAsset, IconError, IconRef, Size, Style};
use crate::generated::Pack;

/// Returns every embedded [`FontAsset`] from enabled packs.
///
/// Register these fonts with your GUI toolkit (for example egui `FontDefinitions`
/// or iced `Font`) before drawing glyphs from [`try_icon`].
#[must_use]
pub fn fonts() -> &'static [FontAsset] {
    crate::generated::fonts()
}

/// Returns the sorted list of icon names available in `pack`.
///
/// Names match the string keys accepted by [`try_icon`].
#[must_use]
pub fn list(pack: Pack) -> &'static [&'static str] {
    crate::generated::list(pack)
}

/// Resolves an icon in `pack` by string `name`, `style`, and `size`.
///
/// On success, returns an [`IconRef`] with the font family and glyph codepoint.
/// Pair the family with bytes from [`fonts`] when configuring your renderer.
///
/// # Errors
///
/// Returns [`IconError`] in these cases:
///
/// - [`IconError::PackDisabled`] — the requested pack feature is not enabled
///   (or no packs are enabled).
/// - [`IconError::IconNotFound`] — `name` is not present in `pack`. The `name`
///   field is a [`std::borrow::Cow`]`<'static, str>` (typically owned for the
///   looked-up string).
/// - [`IconError::VariantUnavailable`] — the icon exists, but `(style, size)`
///   is not among `available`. The `name` field is a
///   [`std::borrow::Cow`]`<'static, str>` (typically borrowed from the pack
///   table).
///
/// # Panics
///
/// Panics only if generated pack tables violate generator invariants: after a
/// variant is confirmed available, the resolve path `expect`s a font family and
/// codepoint for that `(style, size)`. Valid committed maps do not hit these
/// branches.
#[must_use = "icon resolution result should be used"]
pub fn try_icon(pack: Pack, name: &str, style: Style, size: Size) -> Result<IconRef, IconError> {
    crate::generated::try_icon(pack, name, style, size)
}

#[cfg(all(test, feature = "pack-bootstrap"))]
mod tests_bootstrap {
    use super::{list, try_icon};
    use crate::core::{IconError, Size, Style};
    use crate::generated::Pack;

    #[test]
    fn list_exposes_icon_names() {
        let names = list(Pack::Bootstrap);
        assert!(names.contains(&"alarm"));
    }

    #[test]
    fn try_icon_resolves_regular_variant() {
        let icon = try_icon(Pack::Bootstrap, "alarm", Style::Regular, Size::Regular).unwrap();
        assert_eq!(icon.family, "Bootstrap Regular");
    }

    #[test]
    fn try_icon_reports_missing_name() {
        let err = try_icon(Pack::Bootstrap, "missing", Style::Regular, Size::Regular).unwrap_err();
        match err {
            IconError::IconNotFound { pack, name } => {
                assert_eq!(pack, "bootstrap");
                assert_eq!(name, "missing");
            }
            other => panic!("Expected IconNotFound, got {other:?}"),
        }
    }

    #[test]
    fn try_icon_reports_unavailable_variant() {
        let err = try_icon(Pack::Bootstrap, "123", Style::Filled, Size::Regular).unwrap_err();
        match err {
            IconError::VariantUnavailable {
                pack,
                name,
                requested,
                available,
            } => {
                assert_eq!(pack, "bootstrap");
                assert_eq!(name, "123");
                assert_eq!(requested, (Style::Filled, Size::Regular));
                assert!(available.contains(&(Style::Regular, Size::Regular)));
                assert!(!available.contains(&(Style::Filled, Size::Regular)));
            }
            other => panic!("Expected VariantUnavailable, got {other:?}"),
        }
    }
}

#[cfg(all(test, feature = "pack-heroicons"))]
mod tests_heroicons {
    use super::{list, try_icon};
    use crate::core::{IconError, Size, Style};
    use crate::generated::Pack;

    #[test]
    fn list_exposes_icon_names() {
        let names = list(Pack::Heroicons);
        assert!(names.contains(&"academic-cap"));
    }

    #[test]
    fn try_icon_resolves_filled_variant() {
        let icon = try_icon(
            Pack::Heroicons,
            "academic-cap",
            Style::Filled,
            Size::Regular,
        )
        .unwrap();
        assert_eq!(icon.family, "Heroicons Filled");
    }

    #[test]
    fn try_icon_reports_unavailable_variant() {
        let err = try_icon(
            Pack::Heroicons,
            "arrow-left-on-rectangle",
            Style::Outline,
            Size::Mini,
        )
        .unwrap_err();
        match err {
            IconError::VariantUnavailable {
                pack,
                name,
                requested,
                available,
            } => {
                assert_eq!(pack, "heroicons");
                assert_eq!(name, "arrow-left-on-rectangle");
                assert_eq!(requested, (Style::Outline, Size::Mini));
                assert!(available.contains(&(Style::Outline, Size::Regular)));
            }
            other => panic!("Expected VariantUnavailable, got {other:?}"),
        }
    }
}
