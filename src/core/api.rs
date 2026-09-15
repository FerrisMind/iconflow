use std::cmp::Ordering;
use std::fmt;

use crate::core::{FontAsset, IconError, IconRef, Size, Style};
use crate::generated::Pack;

#[cfg(any(
    feature = "pack-bootstrap",
    feature = "pack-carbon",
    feature = "pack-devicon",
    feature = "pack-feather",
    feature = "pack-fluentui",
    feature = "pack-heroicons",
    feature = "pack-iconoir",
    feature = "pack-ionicons",
    feature = "pack-lobe",
    feature = "pack-lucide",
    feature = "pack-octicons",
    feature = "pack-phosphor",
    feature = "pack-remixicon",
    feature = "pack-tabler"
))]
impl fmt::Display for Pack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            #[cfg(feature = "pack-bootstrap")]
            Pack::Bootstrap => f.write_str("bootstrap"),
            #[cfg(feature = "pack-carbon")]
            Pack::Carbon => f.write_str("carbon"),
            #[cfg(feature = "pack-devicon")]
            Pack::Devicon => f.write_str("devicon"),
            #[cfg(feature = "pack-feather")]
            Pack::Feather => f.write_str("feather"),
            #[cfg(feature = "pack-fluentui")]
            Pack::FluentUi => f.write_str("fluentui"),
            #[cfg(feature = "pack-heroicons")]
            Pack::Heroicons => f.write_str("heroicons"),
            #[cfg(feature = "pack-iconoir")]
            Pack::Iconoir => f.write_str("iconoir"),
            #[cfg(feature = "pack-ionicons")]
            Pack::Ionicons => f.write_str("ionicons"),
            #[cfg(feature = "pack-lobe")]
            Pack::Lobe => f.write_str("lobe"),
            #[cfg(feature = "pack-lucide")]
            Pack::Lucide => f.write_str("lucide"),
            #[cfg(feature = "pack-octicons")]
            Pack::Octicons => f.write_str("octicons"),
            #[cfg(feature = "pack-phosphor")]
            Pack::Phosphor => f.write_str("phosphor"),
            #[cfg(feature = "pack-remixicon")]
            Pack::Remixicon => f.write_str("remixicon"),
            #[cfg(feature = "pack-tabler")]
            Pack::Tabler => f.write_str("tabler"),
        }
    }
}

#[cfg(not(any(
    feature = "pack-bootstrap",
    feature = "pack-carbon",
    feature = "pack-devicon",
    feature = "pack-feather",
    feature = "pack-fluentui",
    feature = "pack-heroicons",
    feature = "pack-iconoir",
    feature = "pack-ionicons",
    feature = "pack-lobe",
    feature = "pack-lucide",
    feature = "pack-octicons",
    feature = "pack-phosphor",
    feature = "pack-remixicon",
    feature = "pack-tabler"
)))]
impl fmt::Display for Pack {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

#[cfg(any(
    feature = "pack-bootstrap",
    feature = "pack-carbon",
    feature = "pack-devicon",
    feature = "pack-feather",
    feature = "pack-fluentui",
    feature = "pack-heroicons",
    feature = "pack-iconoir",
    feature = "pack-ionicons",
    feature = "pack-lobe",
    feature = "pack-lucide",
    feature = "pack-octicons",
    feature = "pack-phosphor",
    feature = "pack-remixicon",
    feature = "pack-tabler"
))]
impl PartialOrd for Pack {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(any(
    feature = "pack-bootstrap",
    feature = "pack-carbon",
    feature = "pack-devicon",
    feature = "pack-feather",
    feature = "pack-fluentui",
    feature = "pack-heroicons",
    feature = "pack-iconoir",
    feature = "pack-ionicons",
    feature = "pack-lobe",
    feature = "pack-lucide",
    feature = "pack-octicons",
    feature = "pack-phosphor",
    feature = "pack-remixicon",
    feature = "pack-tabler"
))]
impl Ord for Pack {
    fn cmp(&self, other: &Self) -> Ordering {
        pack_ord_key(*self).cmp(&pack_ord_key(*other))
    }
}

#[cfg(not(any(
    feature = "pack-bootstrap",
    feature = "pack-carbon",
    feature = "pack-devicon",
    feature = "pack-feather",
    feature = "pack-fluentui",
    feature = "pack-heroicons",
    feature = "pack-iconoir",
    feature = "pack-ionicons",
    feature = "pack-lobe",
    feature = "pack-lucide",
    feature = "pack-octicons",
    feature = "pack-phosphor",
    feature = "pack-remixicon",
    feature = "pack-tabler"
)))]
impl PartialOrd for Pack {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(not(any(
    feature = "pack-bootstrap",
    feature = "pack-carbon",
    feature = "pack-devicon",
    feature = "pack-feather",
    feature = "pack-fluentui",
    feature = "pack-heroicons",
    feature = "pack-iconoir",
    feature = "pack-ionicons",
    feature = "pack-lobe",
    feature = "pack-lucide",
    feature = "pack-octicons",
    feature = "pack-phosphor",
    feature = "pack-remixicon",
    feature = "pack-tabler"
)))]
impl Ord for Pack {
    fn cmp(&self, _other: &Self) -> Ordering {
        match *self {}
    }
}

#[cfg(any(
    feature = "pack-bootstrap",
    feature = "pack-carbon",
    feature = "pack-devicon",
    feature = "pack-feather",
    feature = "pack-fluentui",
    feature = "pack-heroicons",
    feature = "pack-iconoir",
    feature = "pack-ionicons",
    feature = "pack-lobe",
    feature = "pack-lucide",
    feature = "pack-octicons",
    feature = "pack-phosphor",
    feature = "pack-remixicon",
    feature = "pack-tabler"
))]
const fn pack_ord_key(pack: Pack) -> u8 {
    match pack {
        #[cfg(feature = "pack-bootstrap")]
        Pack::Bootstrap => 0,
        #[cfg(feature = "pack-carbon")]
        Pack::Carbon => 1,
        #[cfg(feature = "pack-devicon")]
        Pack::Devicon => 2,
        #[cfg(feature = "pack-feather")]
        Pack::Feather => 3,
        #[cfg(feature = "pack-fluentui")]
        Pack::FluentUi => 4,
        #[cfg(feature = "pack-heroicons")]
        Pack::Heroicons => 5,
        #[cfg(feature = "pack-iconoir")]
        Pack::Iconoir => 6,
        #[cfg(feature = "pack-ionicons")]
        Pack::Ionicons => 7,
        #[cfg(feature = "pack-lobe")]
        Pack::Lobe => 8,
        #[cfg(feature = "pack-lucide")]
        Pack::Lucide => 9,
        #[cfg(feature = "pack-octicons")]
        Pack::Octicons => 10,
        #[cfg(feature = "pack-phosphor")]
        Pack::Phosphor => 11,
        #[cfg(feature = "pack-remixicon")]
        Pack::Remixicon => 12,
        #[cfg(feature = "pack-tabler")]
        Pack::Tabler => 13,
    }
}

/// Returns every embedded [`FontAsset`] from enabled packs.
///
/// Register these fonts with your GUI toolkit (for example egui `FontDefinitions`
/// or iced `Font`) before drawing glyphs from [`try_icon`].
#[must_use]
#[inline]
pub fn fonts() -> &'static [FontAsset] {
    crate::generated::fonts()
}

/// Returns the sorted list of icon names available in `pack`.
///
/// Names match the string keys accepted by [`try_icon`].
#[must_use]
#[inline]
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
#[inline]
pub fn try_icon(pack: Pack, name: &str, style: Style, size: Size) -> Result<IconRef, IconError> {
    crate::generated::try_icon(pack, name, style, size)
}

/// Resolves **every** icon in `pack` for `(style, size)` in table order.
///
/// Order matches [`list`]: `resolve_all(pack, …)[i]` corresponds to `list(pack)[i]`.
/// This walks the pack’s entry table once (no per-name binary search), which is the
/// right cold path for icon pickers — one pass beats N× [`try_icon`].
///
/// For warm **positional** frames (picker grids indexed by row/column), keep the
/// returned dense `Vec` and index by position; that is the fastest hot path.
/// Building a `HashMap<&str, IconRef>` from the grid (or via repeated inserts) is
/// expensive relative to one `resolve_all` pass (~order-of-magnitude slower to
/// *build*). Once built, **name lookup** in that map is cheaper than binary search
/// via [`list`] + [`try_icon`] (~5× in R4 measurements). Prefer a HashMap only when
/// the UI’s hot path is name-keyed and the map is built once — not unconditionally.
///
/// # Errors
///
/// This function returns a [`Vec`], not [`Result`]: failures are stored **in-place**
/// as `Err` elements. Typical per-element errors:
///
/// - [`IconError::VariantUnavailable`] — the icon exists in `pack`, but `(style, size)`
///   is not among `available`. The `name` field is a
///   [`std::borrow::Cow`]`<'static, str>` (typically borrowed from the pack table).
///
/// [`IconError::IconNotFound`] does not appear here (every table entry is visited).
/// [`IconError::PackDisabled`] is not produced on this path when pack features are
/// enabled (`Pack` only exposes enabled variants).
///
/// # Panics
///
/// Panics only if generated pack tables violate generator invariants: after a
/// variant is confirmed available, the resolve path `expect`s a font family and
/// codepoint for that `(style, size)`. Valid committed maps do not hit these
/// branches. Same `expect` class as [`try_icon`].
///
/// # Examples
///
/// Resolve a full Bootstrap grid for a picker (requires `pack-bootstrap`):
///
/// ```
/// # #[cfg(feature = "pack-bootstrap")]
/// # fn example() {
/// use iconflow::{list, resolve_all, Pack, Size, Style};
///
/// let names = list(Pack::Bootstrap);
/// let grid = resolve_all(Pack::Bootstrap, Style::Regular, Size::Regular);
/// assert_eq!(grid.len(), names.len());
///
/// let alarm = grid
///     .iter()
///     .zip(names)
///     .find(|(_, name)| **name == "alarm")
///     .and_then(|(slot, _)| slot.as_ref().ok())
///     .expect("alarm Regular/Regular");
/// assert_eq!(alarm.family, "Bootstrap Regular");
/// # }
/// # #[cfg(feature = "pack-bootstrap")]
/// # example();
/// ```
#[must_use = "resolved icon grid should be used"]
pub fn resolve_all(pack: Pack, style: Style, size: Size) -> Vec<Result<IconRef, IconError>> {
    crate::generated::resolve_all(pack, style, size)
}

#[cfg(all(test, feature = "pack-bootstrap"))]
mod tests_bootstrap {
    use super::{list, resolve_all, try_icon};
    use crate::core::{IconError, Size, Style};
    use crate::generated::Pack;

    #[test]
    fn list_exposes_icon_names() {
        let names = list(Pack::Bootstrap);
        assert!(names.contains(&"alarm"));
    }

    #[test]
    fn resolve_all_matches_list_order_and_try_icon() {
        let names = list(Pack::Bootstrap);
        let grid = resolve_all(Pack::Bootstrap, Style::Regular, Size::Regular);
        assert_eq!(grid.len(), names.len());
        let alarm = try_icon(Pack::Bootstrap, "alarm", Style::Regular, Size::Regular).unwrap();
        let idx = names.iter().position(|n| *n == "alarm").unwrap();
        assert_eq!(grid[idx].as_ref().unwrap(), &alarm);
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

#[cfg(all(test, feature = "pack-phosphor"))]
mod tests_phosphor {
    use super::{list, try_icon};
    use crate::core::{IconError, Size, Style};
    use crate::generated::Pack;

    /// Canonical names of the 18 base icons that IcoMoon alias concatenation
    /// previously mangled (PR #2): aliases stored as `"primary, alias"` in
    /// `selection.json` were glued into a single kebab name, so these
    /// canonical names could not resolve.
    const ALIAS_FIX_NAMES: &[&str] = &[
        "asclepius",
        "box-arrow-down",
        "file-dashed",
        "file-magnifying-glass",
        "folder",
        "folder-dashed",
        "folder-minus",
        "folder-open",
        "folder-plus",
        "folder-simple-dashed",
        "infinity",
        "pulse",
        "seal",
        "seal-check",
        "seal-question",
        "seal-warning",
        "text-b",
        "tray-arrow-down",
    ];

    /// All 6 phosphor styles, matching the map's `variants[].style` values
    /// (`bold`/`duotone`/`fill`/`light`/`regular`/`thin`) to `Style`.
    const PHOSPHOR_STYLES: &[Style] = &[
        Style::Bold,
        Style::Duotone,
        Style::Filled,
        Style::Light,
        Style::Regular,
        Style::Thin,
    ];

    /// Canonical registered name for `(name, style)`: phosphor maps expose the
    /// base name for `regular` and a per-style name (`folder-open-bold`,
    /// `asclepius-fill`, ...) for every other style.
    fn phosphor_icon_name(name: &str, style: Style) -> String {
        match style {
            Style::Regular => name.to_owned(),
            Style::Bold => format!("{name}-bold"),
            Style::Duotone => format!("{name}-duotone"),
            Style::Filled => format!("{name}-fill"),
            Style::Light => format!("{name}-light"),
            Style::Thin => format!("{name}-thin"),
            other => panic!("Unexpected phosphor style {other:?}"),
        }
    }

    fn phosphor_family(style: Style) -> &'static str {
        match style {
            Style::Bold => "Phosphor Bold",
            Style::Duotone => "Phosphor Duotone",
            Style::Filled => "Phosphor Filled",
            Style::Light => "Phosphor Light",
            Style::Regular => "Phosphor Regular",
            Style::Thin => "Phosphor Thin",
            other => panic!("Unexpected phosphor style {other:?}"),
        }
    }

    #[test]
    fn list_exposes_icon_names() {
        let names = list(Pack::Phosphor);
        assert!(names.contains(&"acorn"));
        assert!(names.contains(&"folder-open"));
    }

    #[test]
    fn try_icon_resolves_alias_fix_names_in_all_styles() {
        for name in ALIAS_FIX_NAMES {
            for style in PHOSPHOR_STYLES {
                let icon_name = phosphor_icon_name(name, *style);
                let icon = try_icon(Pack::Phosphor, &icon_name, *style, Size::Regular)
                    .unwrap_or_else(|err| {
                        panic!("Expected {icon_name:?} in {style:?} to resolve, got {err:?}")
                    });
                assert_eq!(icon.family, phosphor_family(*style));
            }
        }
    }

    #[test]
    fn try_icon_reports_glued_alias_names_missing() {
        for name in [
            "folder-open-folder-notch-open",
            "pulse-activity",
            "infinity-lemniscate",
            "seal-check-circle-wavy-check",
        ] {
            let err = try_icon(Pack::Phosphor, name, Style::Regular, Size::Regular).unwrap_err();
            match err {
                IconError::IconNotFound {
                    pack,
                    name: missing,
                } => {
                    assert_eq!(pack, "phosphor");
                    assert_eq!(missing, name);
                }
                other => panic!("Expected IconNotFound, got {other:?}"),
            }
        }
    }
}
