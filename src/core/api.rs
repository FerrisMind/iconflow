use crate::core::{FontAsset, IconError, IconRef, Size, Style};
use crate::generated::Pack;

pub fn fonts() -> &'static [FontAsset] {
    crate::generated::fonts()
}

pub fn list(pack: Pack) -> &'static [&'static str] {
    crate::generated::list(pack)
}

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
