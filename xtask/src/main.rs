use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde::de::{self, Visitor};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "PascalCase")]
enum Style {
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

impl Style {
    fn as_rust(self) -> &'static str {
        match self {
            Style::Regular => "Regular",
            Style::Filled => "Filled",
            Style::Outline => "Outline",
            Style::Light => "Light",
            Style::Thin => "Thin",
            Style::Bold => "Bold",
            Style::Duotone => "Duotone",
            Style::Glyph => "Glyph",
            Style::Sharp => "Sharp",
            Style::Rounded => "Rounded",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Size {
    Tiny,
    Mini,
    Regular,
    Large,
    Custom(u16),
}

impl Size {
    fn rust_expr(self) -> String {
        match self {
            Size::Tiny => "Size::Tiny".to_string(),
            Size::Mini => "Size::Mini".to_string(),
            Size::Regular => "Size::Regular".to_string(),
            Size::Large => "Size::Large".to_string(),
            Size::Custom(value) => format!("Size::Custom({value})"),
        }
    }
}

impl<'de> Deserialize<'de> for Size {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SizeVisitor;

        impl<'de> Visitor<'de> for SizeVisitor {
            type Value = Size;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a size string (Tiny/Mini/Regular/Large) or a positive integer")
            }

            fn visit_str<E>(self, value: &str) -> Result<Size, E>
            where
                E: de::Error,
            {
                match value {
                    "Tiny" => Ok(Size::Tiny),
                    "Mini" => Ok(Size::Mini),
                    "Regular" => Ok(Size::Regular),
                    "Large" => Ok(Size::Large),
                    _ => Err(E::unknown_variant(
                        value,
                        &["Tiny", "Mini", "Regular", "Large"],
                    )),
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Size, E>
            where
                E: de::Error,
            {
                if value == 0 || value > u16::MAX as u64 {
                    return Err(E::custom("custom size must be between 1 and 65535"));
                }
                Ok(Size::Custom(value as u16))
            }
        }

        deserializer.deserialize_any(SizeVisitor)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct VariantKey {
    style: Style,
    size: Size,
}

#[derive(Debug, Deserialize)]
struct PackMap {
    pack_id: String,
    variants: Vec<Variant>,
    icons: Vec<Icon>,
    #[serde(skip)]
    source_path: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
struct Variant {
    id: String,
    style: Style,
    size: Size,
    family: String,
    ttf_asset_path: String,
    #[serde(default)]
    feature: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Icon {
    name: String,
    codepoint: Option<u32>,
    #[serde(default)]
    overrides: BTreeMap<String, u32>,
    #[serde(default)]
    availability: Option<Vec<String>>,
}

#[derive(Debug)]
struct VariantInfo {
    id: String,
    key: VariantKey,
    family: String,
    ttf_asset_path: String,
    feature: Option<String>,
}

#[derive(Debug)]
struct NormalizedIcon {
    name: String,
    ident: String,
    codepoints: Vec<(VariantKey, u32)>,
}

#[derive(Debug)]
struct NormalizedPack {
    pack_id: String,
    variants: Vec<VariantInfo>,
    icons: Vec<NormalizedIcon>,
}

#[derive(Debug)]
struct FontAssetInfo {
    const_ident: String,
    family: String,
    ttf_asset_path: String,
    feature: Option<String>,
}

type FontAssetCollection = (
    Vec<FontAssetInfo>,
    BTreeMap<String, String>,
    BTreeMap<VariantKey, Option<String>>,
);

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        print_usage();
        return Ok(());
    };

    match cmd.as_str() {
        "gen" => {
            let mut check = false;
            for arg in args {
                match arg.as_str() {
                    "--check" => check = true,
                    _ => bail!("Unknown argument: {arg}"),
                }
            }
            run_gen(check)
        }
        _ => {
            print_usage();
            bail!("Unknown command: {cmd}")
        }
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  cargo xtask gen [--check]");
}

fn run_gen(check: bool) -> Result<()> {
    let repo_root = repo_root()?;
    let maps_dir = repo_root.join("assets").join("maps");
    let generated_dir = repo_root.join("src").join("generated");

    let mut map_paths: Vec<PathBuf> = fs::read_dir(&maps_dir)
        .with_context(|| format!("Reading maps directory {maps_dir:?}"))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().map(|ext| ext == "json").unwrap_or(false))
        .collect();
    map_paths.sort();

    if map_paths.is_empty() {
        bail!("No map files found in {maps_dir:?}");
    }

    let mut packs = Vec::new();
    for path in map_paths {
        packs.push(load_pack_map(&path)?);
    }

    let mut normalized = Vec::new();
    for pack in packs {
        normalized.push(normalize_pack(pack)?);
    }
    normalized.sort_by(|a, b| a.pack_id.cmp(&b.pack_id));

    // F-016: uniqueness after normalize+sort
    let mut seen = BTreeSet::new();
    for p in &normalized {
        if !seen.insert(p.pack_id.clone()) {
            bail!("duplicate pack_id '{}'", p.pack_id);
        }
    }

    // F-017: bidirectional pack-* feature check against repo-root Cargo.toml
    check_pack_features(&repo_root, &normalized)?;

    let mut outputs = Vec::new();
    outputs.push((
        generated_dir.join("mod.rs"),
        rustfmt(&render_mod(&normalized)?)?,
    ));

    for pack in &normalized {
        let path = generated_dir.join(format!("{}.rs", pack.pack_id));
        outputs.push((path, rustfmt(&render_pack(pack)?)?));
    }

    // R3-N-03: fail on unexpected src/generated/*.rs (check and write modes).
    let expected_names = expected_generated_rs_names(&normalized);
    check_no_orphan_generated(&generated_dir, &expected_names)?;

    for (path, content) in &outputs {
        write_output(path, content, check)?;
    }

    Ok(())
}

/// Filenames that `gen` is allowed to emit under `src/generated/`.
fn expected_generated_rs_names(packs: &[NormalizedPack]) -> BTreeSet<String> {
    let mut expected = BTreeSet::new();
    expected.insert("mod.rs".to_string());
    for pack in packs {
        expected.insert(format!("{}.rs", pack.pack_id));
    }
    expected
}

/// Return sorted orphan basenames: on-disk `.rs` names not in `expected`.
fn orphan_generated_rs_names(
    expected: &BTreeSet<String>,
    on_disk_names: impl IntoIterator<Item = String>,
) -> Vec<String> {
    let mut orphans: Vec<String> = on_disk_names
        .into_iter()
        .filter(|name| !expected.contains(name))
        .collect();
    orphans.sort();
    orphans
}

fn check_no_orphan_generated(generated_dir: &Path, expected: &BTreeSet<String>) -> Result<()> {
    let entries = match fs::read_dir(generated_dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            // Write mode creates the dir later; nothing on disk means no orphans.
            return Ok(());
        }
        Err(err) => {
            return Err(err).with_context(|| {
                format!("Reading generated directory {}", generated_dir.display())
            });
        }
    };

    let mut on_disk = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| {
            format!(
                "Reading entry in generated directory {}",
                generated_dir.display()
            )
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            bail!(
                "Non-UTF-8 generated filename under {}: {}",
                generated_dir.display(),
                path.display()
            );
        };
        on_disk.push(name.to_string());
    }

    let orphans = orphan_generated_rs_names(expected, on_disk);
    if !orphans.is_empty() {
        bail!(
            "Unexpected generated .rs files (orphans) in {}: {orphans:?}. \
             Remove them manually or update pack maps; gen does not auto-delete.",
            generated_dir.display()
        );
    }
    Ok(())
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .context("xtask is expected to live one level below repo root")
        .map(Path::to_path_buf)
}

fn check_pack_features(repo_root: &Path, packs: &[NormalizedPack]) -> Result<()> {
    let cargo_toml_path = repo_root.join("Cargo.toml");
    let raw = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Reading {}", cargo_toml_path.display()))?;
    let manifest: toml::Value = raw
        .parse()
        .with_context(|| format!("Parsing {}", cargo_toml_path.display()))?;

    let features = manifest
        .get("features")
        .and_then(|v| v.as_table())
        .context("Cargo.toml missing [features] table")?;

    let feature_packs: BTreeSet<String> = features
        .keys()
        .filter(|k| k.starts_with("pack-"))
        .cloned()
        .collect();

    let map_packs: BTreeSet<String> = packs
        .iter()
        .map(|p| format!("pack-{}", p.pack_id))
        .collect();

    if !map_packs.is_subset(&feature_packs) {
        let missing: Vec<&String> = map_packs.difference(&feature_packs).collect();
        bail!("pack maps not subset of Cargo.toml pack-* features; missing features: {missing:?}");
    }
    if !feature_packs.is_subset(&map_packs) {
        let extra: Vec<&String> = feature_packs.difference(&map_packs).collect();
        bail!("Cargo.toml pack-* features not subset of pack maps; extra features: {extra:?}");
    }

    // N-G-003: every non-empty variant.feature must exist in [features]
    for pack in packs {
        for variant in &pack.variants {
            if let Some(feature) = &variant.feature {
                if feature.is_empty() {
                    bail!(
                        "pack '{}': variant '{}' has empty feature string",
                        pack.pack_id,
                        variant.id
                    );
                }
                if !features.contains_key(feature) {
                    bail!(
                        "pack '{}': variant.feature '{}' not found in Cargo.toml [features]",
                        pack.pack_id,
                        feature
                    );
                }
            }
        }
    }

    Ok(())
}

fn load_pack_map(path: &Path) -> Result<PackMap> {
    let raw = fs::read_to_string(path).with_context(|| format!("Reading {path:?}"))?;
    let mut map: PackMap =
        serde_json::from_str(&raw).with_context(|| format!("Parsing JSON in {path:?}"))?;
    map.source_path = path.to_path_buf();
    Ok(map)
}

fn normalize_pack(pack: PackMap) -> Result<NormalizedPack> {
    let mut variants = pack.variants.clone();
    variants.sort_by(|a, b| a.id.cmp(&b.id));

    let mut seen_variant_ids = BTreeSet::new();
    let mut seen_variant_keys = BTreeSet::new();
    let mut variants_info = Vec::new();
    let mut variant_key_by_id = BTreeMap::new();

    for variant in variants {
        if !seen_variant_ids.insert(variant.id.clone()) {
            bail!(
                "{}: duplicate variant.id '{}'",
                pack.source_path.display(),
                variant.id
            );
        }

        let key = VariantKey {
            style: variant.style,
            size: variant.size,
        };
        if !seen_variant_keys.insert(key) {
            bail!(
                "{}: duplicate variant style/size {:?}/{:?}",
                pack.source_path.display(),
                variant.style,
                variant.size
            );
        }

        if let Some(feature) = &variant.feature
            && feature.trim().is_empty()
        {
            bail!(
                "{}: variant '{}' has empty feature name",
                pack.source_path.display(),
                variant.id
            );
        }
        variant_key_by_id.insert(variant.id.clone(), key);
        variants_info.push(VariantInfo {
            id: variant.id,
            key,
            family: variant.family,
            ttf_asset_path: variant.ttf_asset_path,
            feature: variant.feature,
        });
    }

    let variant_ids: Vec<String> = variants_info.iter().map(|v| v.id.clone()).collect();
    let variant_id_set: BTreeSet<&str> = variants_info.iter().map(|v| v.id.as_str()).collect();

    let mut seen_icon_names = BTreeSet::new();
    let mut seen_icon_idents = BTreeMap::new();
    let mut icons_info = Vec::new();

    for icon in &pack.icons {
        if !seen_icon_names.insert(icon.name.clone()) {
            bail!(
                "{}: duplicate icon.name '{}'",
                pack.source_path.display(),
                icon.name
            );
        }

        let ident = normalize_icon_name(&icon.name)?;
        if let Some(prev) = seen_icon_idents.insert(ident.clone(), icon.name.clone()) {
            bail!(
                "{}: icon name collision: '{}' and '{}' both map to '{}'",
                pack.source_path.display(),
                prev,
                icon.name,
                ident
            );
        }

        for variant_id in icon.overrides.keys() {
            if !variant_id_set.contains(variant_id.as_str()) {
                bail!(
                    "{}: icon '{}' overrides unknown variant '{}'",
                    pack.source_path.display(),
                    icon.name,
                    variant_id
                );
            }
        }

        if let Some(availability) = &icon.availability {
            for variant_id in availability {
                if !variant_id_set.contains(variant_id.as_str()) {
                    bail!(
                        "{}: icon '{}' availability unknown variant '{}'",
                        pack.source_path.display(),
                        icon.name,
                        variant_id
                    );
                }
            }
            if !icon.overrides.is_empty() {
                for variant_id in icon.overrides.keys() {
                    if !availability.iter().any(|id| id == variant_id) {
                        bail!(
                            "{}: icon '{}' overrides not listed in availability: '{}'",
                            pack.source_path.display(),
                            icon.name,
                            variant_id
                        );
                    }
                }
            }
        }

        let availability = match &icon.availability {
            Some(list) => {
                if list.is_empty() {
                    bail!(
                        "{}: icon '{}' availability is empty",
                        pack.source_path.display(),
                        icon.name
                    );
                }
                let mut dedup = BTreeSet::new();
                for item in list {
                    if !dedup.insert(item.as_str()) {
                        bail!(
                            "{}: icon '{}' availability has duplicates: '{}'",
                            pack.source_path.display(),
                            icon.name,
                            item
                        );
                    }
                }
                list.clone()
            }
            None => {
                if icon.codepoint.is_some() {
                    variant_ids.clone()
                } else if !icon.overrides.is_empty() {
                    icon.overrides.keys().cloned().collect()
                } else {
                    bail!(
                        "{}: icon '{}' has no codepoint or overrides",
                        pack.source_path.display(),
                        icon.name
                    );
                }
            }
        };

        let availability_set: BTreeSet<&str> = availability.iter().map(|id| id.as_str()).collect();
        let mut codepoints = Vec::new();

        for variant_id in &variant_ids {
            if !availability_set.contains(variant_id.as_str()) {
                continue;
            }

            let codepoint = match icon.overrides.get(variant_id) {
                Some(value) => *value,
                None => icon.codepoint.ok_or_else(|| {
                    anyhow::anyhow!(
                        "{}: icon '{}' missing codepoint for variant '{}'",
                        pack.source_path.display(),
                        icon.name,
                        variant_id
                    )
                })?,
            };

            let key = *variant_key_by_id.get(variant_id).ok_or_else(|| {
                anyhow::anyhow!(
                    "{}: icon '{}' references unknown variant '{}'",
                    pack.source_path.display(),
                    icon.name,
                    variant_id
                )
            })?;

            codepoints.push((key, codepoint));
        }

        if codepoints.is_empty() {
            bail!(
                "{}: icon '{}' has no available variants",
                pack.source_path.display(),
                icon.name
            );
        }

        icons_info.push(NormalizedIcon {
            name: icon.name.clone(),
            ident,
            codepoints,
        });
    }

    icons_info.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(NormalizedPack {
        pack_id: pack.pack_id,
        variants: variants_info,
        icons: icons_info,
    })
}

fn collect_font_assets(pack: &NormalizedPack) -> Result<FontAssetCollection> {
    let mut asset_feature_sets: BTreeMap<String, BTreeSet<Option<String>>> = BTreeMap::new();
    let mut asset_families: BTreeMap<String, String> = BTreeMap::new();
    let mut variant_feature_by_key = BTreeMap::new();

    for variant in &pack.variants {
        let path = variant.ttf_asset_path.replace('\\', "/");
        variant_feature_by_key.insert(variant.key, variant.feature.clone());
        asset_feature_sets
            .entry(path.clone())
            .or_default()
            .insert(variant.feature.clone());
        if let Some(existing) = asset_families.get(&path) {
            if existing != &variant.family {
                bail!(
                    "Pack {} has conflicting family names for {}: '{}' vs '{}'",
                    pack.pack_id,
                    path,
                    existing,
                    variant.family
                );
            }
        } else {
            asset_families.insert(path.clone(), variant.family.clone());
        }
    }

    let mut assets = Vec::new();
    let mut asset_const_by_path = BTreeMap::new();
    for (path, family) in asset_families {
        let const_ident = font_asset_const_ident_from_path(&pack.pack_id, &path)?;
        let feature_set = asset_feature_sets.get(&path).cloned().unwrap_or_default();
        let feature = if feature_set.len() == 1 {
            feature_set.into_iter().next().unwrap_or(None)
        } else {
            None
        };
        asset_const_by_path.insert(path.clone(), const_ident.clone());
        assets.push(FontAssetInfo {
            const_ident,
            family,
            ttf_asset_path: path,
            feature,
        });
    }

    Ok((assets, asset_const_by_path, variant_feature_by_key))
}

fn render_mod(packs: &[NormalizedPack]) -> Result<String> {
    let mut out = String::new();
    push_line(&mut out, "// @generated by xtask gen. DO NOT EDIT.");
    push_line(&mut out, "");
    push_line(
        &mut out,
        "use crate::core::{FontAsset, IconError, IconRef, Size, Style};",
    );
    push_line(&mut out, "");

    for pack in packs {
        let pack_id = &pack.pack_id;
        push_line(&mut out, &format!("#[cfg(feature = \"pack-{pack_id}\")]"));
        push_line(&mut out, &format!("pub(crate) mod {pack_id};"));
        push_line(&mut out, "");
    }

    push_line(
        &mut out,
        "/// Icon pack selectable via [`crate::try_icon`] / [`crate::list`] / [`crate::resolve_all`].",
    );
    push_line(&mut out, "///");
    push_line(
        &mut out,
        "/// Variants exist only when the corresponding `pack-*` Cargo feature is enabled.",
    );
    push_line(
        &mut out,
        "#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]",
    );
    push_line(&mut out, "pub enum Pack {");
    for pack in packs {
        let pack_id = &pack.pack_id;
        let ident = pack_enum_ident(pack_id)?;
        push_line(
            &mut out,
            &format!(
                "    /// {} (`pack-{pack_id}`).",
                pack_enum_doc_label(pack_id)
            ),
        );
        push_line(
            &mut out,
            &format!("    #[cfg(feature = \"pack-{pack_id}\")]"),
        );
        push_line(&mut out, &format!("    {ident},"));
    }
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, "pub fn fonts() -> &'static [FontAsset] {");
    push_line(&mut out, "    &[");
    for pack in packs {
        let pack_id = &pack.pack_id;
        let (assets, _, _) = collect_font_assets(pack)?;
        for asset in assets {
            push_line(
                &mut out,
                &cfg_pack_feature_line(pack_id, asset.feature.as_deref(), 8),
            );
            push_line(
                &mut out,
                &format!("        {pack_id}::{},", asset.const_ident),
            );
        }
    }
    push_line(&mut out, "    ]");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    let pack_feature_list: Vec<String> = packs
        .iter()
        .map(|pack| format!("feature = \"pack-{}\"", pack.pack_id))
        .collect();
    let any_packs_cfg = pack_feature_list.join(", ");

    push_line(&mut out, &format!("#[cfg(any({any_packs_cfg}))]"));
    push_line(
        &mut out,
        "pub fn list(pack: Pack) -> &'static [&'static str] {",
    );
    push_line(&mut out, "    match pack {");
    for pack in packs {
        let pack_id = &pack.pack_id;
        let ident = pack_enum_ident(pack_id)?;
        push_line(
            &mut out,
            &format!("        #[cfg(feature = \"pack-{pack_id}\")]"),
        );
        push_line(
            &mut out,
            &format!("        Pack::{ident} => {pack_id}::ICON_NAMES,"),
        );
    }
    push_line(&mut out, "    }");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, &format!("#[cfg(not(any({any_packs_cfg})))]"));
    push_line(
        &mut out,
        "pub fn list(_pack: Pack) -> &'static [&'static str] {",
    );
    push_line(&mut out, "    &[]");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, &format!("#[cfg(any({any_packs_cfg}))]"));
    push_line(
        &mut out,
        "pub fn try_icon(pack: Pack, name: &str, style: Style, size: Size) -> Result<IconRef, IconError> {",
    );
    push_line(&mut out, "    match pack {");
    for pack in packs {
        let pack_id = &pack.pack_id;
        let ident = pack_enum_ident(pack_id)?;
        push_line(
            &mut out,
            &format!("        #[cfg(feature = \"pack-{pack_id}\")]"),
        );
        push_line(
            &mut out,
            &format!("        Pack::{ident} => match {pack_id}::icon_entry(name) {{"),
        );
        push_line(
            &mut out,
            "            None => Err(IconError::IconNotFound {",
        );
        push_line(
            &mut out,
            &format!("                pack: {pack_id}::PACK_ID,"),
        );
        push_line(
            &mut out,
            "                name: ::std::borrow::Cow::Owned(name.to_owned()),",
        );
        push_line(&mut out, "            }),");
        push_line(&mut out, "            Some(entry) => resolve_found(");
        push_line(&mut out, &format!("                {pack_id}::PACK_ID,"));
        push_line(&mut out, "                entry,");
        push_line(&mut out, "                style,");
        push_line(&mut out, "                size,");
        push_line(
            &mut out,
            &format!(
                "                {pack_id}::variant_info(style, size).map(|info| info.family),"
            ),
        );
        push_line(&mut out, "            ),");
        push_line(&mut out, "        },");
    }
    push_line(&mut out, "    }");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, &format!("#[cfg(not(any({any_packs_cfg})))]"));
    push_line(
        &mut out,
        "pub fn try_icon(_pack: Pack, _name: &str, _style: Style, _size: Size) -> Result<IconRef, IconError> {",
    );
    push_line(
        &mut out,
        "    Err(IconError::PackDisabled { pack: \"none\" })",
    );
    push_line(&mut out, "}");
    push_line(&mut out, "");

    // R3-N-09: linear table walk for picker cold / dense warm caches (no per-name binary search).
    push_line(&mut out, &format!("#[cfg(any({any_packs_cfg}))]"));
    push_line(
        &mut out,
        "pub fn resolve_all(pack: Pack, style: Style, size: Size) -> Vec<Result<IconRef, IconError>> {",
    );
    push_line(&mut out, "    match pack {");
    for pack in packs {
        let pack_id = &pack.pack_id;
        let ident = pack_enum_ident(pack_id)?;
        push_line(
            &mut out,
            &format!("        #[cfg(feature = \"pack-{pack_id}\")]"),
        );
        push_line(&mut out, &format!("        Pack::{ident} => {{"));
        push_line(
            &mut out,
            &format!(
                "            let family = {pack_id}::variant_info(style, size).map(|info| info.family);"
            ),
        );
        push_line(
            &mut out,
            &format!(
                "            let mut out = Vec::with_capacity({pack_id}::ICON_ENTRIES.len());"
            ),
        );
        push_line(
            &mut out,
            &format!("            for entry in {pack_id}::ICON_ENTRIES {{"),
        );
        push_line(&mut out, "                out.push(resolve_found(");
        push_line(&mut out, &format!("                    {pack_id}::PACK_ID,"));
        push_line(&mut out, "                    entry,");
        push_line(&mut out, "                    style,");
        push_line(&mut out, "                    size,");
        push_line(&mut out, "                    family,");
        push_line(&mut out, "                ));");
        push_line(&mut out, "            }");
        push_line(&mut out, "            out");
        push_line(&mut out, "        }");
    }
    push_line(&mut out, "    }");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, &format!("#[cfg(not(any({any_packs_cfg})))]"));
    push_line(
        &mut out,
        "pub fn resolve_all(_pack: Pack, _style: Style, _size: Size) -> Vec<Result<IconRef, IconError>> {",
    );
    push_line(&mut out, "    Vec::new()");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, &format!("#[cfg(any({any_packs_cfg}))]"));
    push_line(&mut out, "fn resolve_found(");
    push_line(&mut out, "    pack: &'static str,");
    push_line(&mut out, "    entry: &'static crate::core::IconEntry,");
    push_line(&mut out, "    style: Style,");
    push_line(&mut out, "    size: Size,");
    push_line(&mut out, "    family: Option<&'static str>,");
    push_line(&mut out, ") -> Result<IconRef, IconError> {");
    push_line(
        &mut out,
        "    if !entry.available.contains(&(style, size)) {",
    );
    push_line(
        &mut out,
        "        return Err(IconError::VariantUnavailable {",
    );
    push_line(&mut out, "            pack,");
    push_line(
        &mut out,
        "            name: ::std::borrow::Cow::Borrowed(entry.name),",
    );
    push_line(&mut out, "            requested: (style, size),");
    push_line(&mut out, "            available: entry.available,");
    push_line(&mut out, "        });");
    push_line(&mut out, "    }");
    // F-026 / D10: outline expect failure paths as cold helpers
    push_line(
        &mut out,
        "    let family = resolve_invariant_family(family);",
    );
    push_line(
        &mut out,
        "    let key = crate::core::VariantKey { style, size };",
    );
    push_line(&mut out, "    let codepoint = resolve_invariant_codepoint(");
    push_line(&mut out, "        entry");
    push_line(&mut out, "            .variants");
    push_line(&mut out, "            .iter()");
    push_line(&mut out, "            .find(|(k, _)| *k == key)");
    push_line(&mut out, "            .map(|(_, cp)| *cp),");
    push_line(&mut out, "    );");
    push_line(&mut out, "    Ok(IconRef { family, codepoint })");
    push_line(&mut out, "}");
    push_line(&mut out, "");
    push_line(&mut out, &format!("#[cfg(any({any_packs_cfg}))]"));
    push_line(&mut out, "#[cold]");
    push_line(&mut out, "#[inline(never)]");
    push_line(
        &mut out,
        "fn resolve_invariant_family(family: Option<&'static str>) -> &'static str {",
    );
    push_line(
        &mut out,
        "    family.expect(\"Icon variant should have a font family\")",
    );
    push_line(&mut out, "}");
    push_line(&mut out, "");
    push_line(&mut out, &format!("#[cfg(any({any_packs_cfg}))]"));
    push_line(&mut out, "#[cold]");
    push_line(&mut out, "#[inline(never)]");
    push_line(
        &mut out,
        "fn resolve_invariant_codepoint(codepoint: Option<u32>) -> u32 {",
    );
    push_line(
        &mut out,
        "    codepoint.expect(\"Icon variant should have a codepoint\")",
    );
    push_line(&mut out, "}");

    Ok(out)
}

fn render_pack(pack: &NormalizedPack) -> Result<String> {
    let mut out = String::new();
    push_line(&mut out, "// @generated by xtask gen. DO NOT EDIT.");
    // N-G-001 / D1: no dead typed Icon surface; no file-level dead_code allows.
    push_line(
        &mut out,
        "use crate::core::{FontAsset, IconEntry, Size, Style, VariantKey};",
    );
    push_line(&mut out, "");
    push_line(
        &mut out,
        &format!("pub(crate) const PACK_ID: &str = \"{}\";", pack.pack_id),
    );
    push_line(&mut out, "");

    let (assets, _asset_const_by_path, variant_feature_by_key) = collect_font_assets(pack)?;

    for asset in &assets {
        if let Some(feature) = &asset.feature {
            push_line(&mut out, &cfg_attr_line(feature, 0));
        }
        push_line(
            &mut out,
            &format!(
                "pub(crate) const {}: FontAsset = FontAsset {{ family: \"{}\", bytes: include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")) }};",
                asset.const_ident, asset.family, asset.ttf_asset_path
            ),
        );
    }

    push_line(&mut out, "");

    push_line(&mut out, "pub(crate) const ICON_NAMES: &[&str] = &[");
    for icon in &pack.icons {
        push_line(&mut out, &format!("    \"{}\",", icon.name));
    }
    push_line(&mut out, "];");
    push_line(&mut out, "");

    push_line(&mut out, "#[derive(Clone, Copy, Debug)]");
    push_line(&mut out, "pub(crate) struct VariantInfo {");
    push_line(&mut out, "    pub key: VariantKey,");
    push_line(&mut out, "    pub family: &'static str,");
    push_line(&mut out, "}");
    push_line(&mut out, "");
    push_line(&mut out, "pub(crate) const VARIANTS: &[VariantInfo] = &[");
    for variant in &pack.variants {
        if let Some(feature) = &variant.feature {
            push_line(&mut out, &cfg_attr_line(feature, 4));
        }
        push_line(
            &mut out,
            &format!(
                "    VariantInfo {{ key: {}, family: \"{}\" }},",
                variant_key_expr(variant.key),
                variant.family
            ),
        );
    }
    push_line(&mut out, "];");
    push_line(&mut out, "");

    for icon in &pack.icons {
        let const_name = icon_codepoints_const_ident(&icon.ident)?;
        push_line(
            &mut out,
            &format!("const {const_name}: &[(VariantKey, u32)] = &["),
        );
        for (key, codepoint) in &icon.codepoints {
            if let Some(feature) = variant_feature_by_key.get(key).and_then(|f| f.as_deref()) {
                push_line(&mut out, &cfg_attr_line(feature, 4));
            }
            push_line(
                &mut out,
                &format!("    ({}, {codepoint}),", variant_key_expr(*key)),
            );
        }
        push_line(&mut out, "];");
        push_line(&mut out, "");
    }

    for icon in &pack.icons {
        let const_name = icon_available_const_ident(&icon.ident)?;
        push_line(
            &mut out,
            &format!("const {const_name}: &[(Style, Size)] = &["),
        );
        for (key, _) in &icon.codepoints {
            if let Some(feature) = variant_feature_by_key.get(key).and_then(|f| f.as_deref()) {
                push_line(&mut out, &cfg_attr_line(feature, 4));
            }
            push_line(
                &mut out,
                &format!(
                    "    (Style::{}, {}),",
                    key.style.as_rust(),
                    key.size.rust_expr()
                ),
            );
        }
        push_line(&mut out, "];");
        push_line(&mut out, "");
    }

    push_line(&mut out, "pub(crate) const ICON_ENTRIES: &[IconEntry] = &[");
    for icon in &pack.icons {
        let codepoints_const = icon_codepoints_const_ident(&icon.ident)?;
        let available_const = icon_available_const_ident(&icon.ident)?;
        push_line(
            &mut out,
            &format!(
                "    IconEntry {{ name: \"{}\", variants: {codepoints_const}, available: {available_const} }},",
                icon.name
            ),
        );
    }
    push_line(&mut out, "];");
    push_line(&mut out, "");

    push_line(
        &mut out,
        "pub(crate) fn variant_info(style: Style, size: Size) -> Option<&'static VariantInfo> {",
    );
    push_line(
        &mut out,
        "    VARIANTS.iter().find(|variant| variant.key == VariantKey { style, size })",
    );
    push_line(&mut out, "}");
    push_line(&mut out, "");
    push_line(
        &mut out,
        "pub(crate) fn icon_entry(name: &str) -> Option<&'static IconEntry> {",
    );
    push_line(
        &mut out,
        "    let idx = ICON_ENTRIES.binary_search_by(|e| e.name.as_bytes().cmp(name.as_bytes())).ok()?;",
    );
    push_line(&mut out, "    Some(&ICON_ENTRIES[idx])");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    // N-G-002 / D8: pack invariants without Icon enum
    push_line(&mut out, "#[cfg(test)]");
    push_line(&mut out, "mod gen_invariants {");
    push_line(&mut out, "    use super::*;");
    push_line(&mut out, "");
    push_line(&mut out, "    #[test]");
    push_line(&mut out, "    fn icon_entries_sorted_by_name() {");
    push_line(
        &mut out,
        "        assert!(ICON_ENTRIES.windows(2).all(|w| w[0].name < w[1].name));",
    );
    push_line(&mut out, "    }");
    push_line(&mut out, "");
    push_line(&mut out, "    #[test]");
    push_line(&mut out, "    fn icon_names_match_entries() {");
    push_line(
        &mut out,
        "        assert_eq!(ICON_NAMES.len(), ICON_ENTRIES.len());",
    );
    push_line(
        &mut out,
        "        for (i, entry) in ICON_ENTRIES.iter().enumerate() {",
    );
    push_line(
        &mut out,
        "            assert_eq!(ICON_NAMES[i], entry.name);",
    );
    push_line(&mut out, "        }");
    push_line(&mut out, "    }");
    push_line(&mut out, "");
    push_line(&mut out, "    #[test]");
    push_line(&mut out, "    fn every_entry_name_resolves() {");
    push_line(&mut out, "        for entry in ICON_ENTRIES {");
    push_line(
        &mut out,
        "            let resolved = icon_entry(entry.name);",
    );
    push_line(
        &mut out,
        "            assert!(resolved.is_some(), \"missing entry for {}\", entry.name);",
    );
    push_line(
        &mut out,
        "            assert_eq!(resolved.unwrap().name, entry.name);",
    );
    push_line(&mut out, "        }");
    push_line(&mut out, "    }");
    push_line(&mut out, "");
    push_line(&mut out, "    #[test]");
    push_line(&mut out, "    fn name_cmp_matches_bytes_cmp() {");
    push_line(&mut out, "        for window in ICON_ENTRIES.windows(2) {");
    push_line(&mut out, "            let a = window[0].name;");
    push_line(&mut out, "            let b = window[1].name;");
    push_line(
        &mut out,
        "            assert_eq!(a.cmp(b), a.as_bytes().cmp(b.as_bytes()));",
    );
    push_line(&mut out, "        }");
    push_line(&mut out, "    }");
    push_line(&mut out, "}");

    Ok(out)
}

fn variant_key_expr(key: VariantKey) -> String {
    format!(
        "VariantKey {{ style: Style::{}, size: {} }}",
        key.style.as_rust(),
        key.size.rust_expr()
    )
}

fn cfg_attr_line(feature: &str, indent: usize) -> String {
    format!(
        "{:indent$}#[cfg(feature = \"{feature}\")]",
        "",
        indent = indent
    )
}

fn cfg_pack_feature_line(pack_id: &str, feature: Option<&str>, indent: usize) -> String {
    match feature {
        Some(feature) => format!(
            "{:indent$}#[cfg(all(feature = \"pack-{pack_id}\", feature = \"{feature}\"))]",
            "",
            indent = indent
        ),
        None => format!(
            "{:indent$}#[cfg(feature = \"pack-{pack_id}\")]",
            "",
            indent = indent
        ),
    }
}

fn font_asset_const_ident_from_path(pack_id: &str, ttf_asset_path: &str) -> Result<String> {
    let path = Path::new(ttf_asset_path);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid ttf asset path: {ttf_asset_path}"))?;
    let normalized = stem.replace('-', "_");
    let stem_ident = to_upper_snake(&normalized)?;
    let pack_ident = to_upper_snake(pack_id)?;
    Ok(format!("FONT_ASSET_{pack_ident}_{stem_ident}"))
}

fn normalize_icon_name(name: &str) -> Result<String> {
    if name.is_empty() {
        bail!("Icon name is empty");
    }

    let mut ident = to_pascal_case(name)?;
    if ident
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        ident = format!("Icon{ident}");
    }

    if is_rust_keyword(&ident) {
        ident.push('_');
    }

    Ok(ident)
}

fn to_pascal_case(name: &str) -> Result<String> {
    let mut out = String::new();
    for part in name.split('-') {
        if part.is_empty() {
            bail!("Icon name contains empty segment: '{name}'");
        }
        let mut chars = part.chars();
        let Some(first) = chars.next() else {
            continue;
        };
        if first.is_ascii_alphabetic() {
            out.push(first.to_ascii_uppercase());
        } else {
            out.push(first);
        }
        out.extend(chars);
    }
    Ok(out)
}

fn is_rust_keyword(ident: &str) -> bool {
    matches!(
        ident.to_ascii_lowercase().as_str(),
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "union"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "try"
            | "yield"
    )
}

fn icon_codepoints_const_ident(ident: &str) -> Result<String> {
    let upper = to_upper_snake(ident)?;
    Ok(format!("ICON_{upper}_CODEPOINTS"))
}

fn icon_available_const_ident(ident: &str) -> Result<String> {
    let upper = to_upper_snake(ident)?;
    Ok(format!("ICON_{upper}_AVAILABLE"))
}

fn to_upper_snake(ident: &str) -> Result<String> {
    if ident.is_empty() {
        bail!("Identifier is empty");
    }
    let mut out = String::new();
    for (idx, ch) in ident.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if idx != 0 {
                out.push('_');
            }
            out.push(ch);
        } else if ch.is_ascii_lowercase() {
            out.push(ch.to_ascii_uppercase());
        } else if ch.is_ascii_digit() {
            if idx != 0 && !out.ends_with('_') {
                out.push('_');
            }
            out.push(ch);
        } else if ch == '_' {
            if !out.ends_with('_') {
                out.push('_');
            }
        } else {
            bail!("Identifier contains unsupported character '{ch}'");
        }
    }
    Ok(out)
}

fn pack_enum_ident(pack_id: &str) -> Result<String> {
    // F-014 / D9: fluentui → FluentUi (not naive PascalCase Fluentui)
    if pack_id == "fluentui" {
        return Ok("FluentUi".to_string());
    }
    let mut ident = to_pascal_case(pack_id)?;
    if is_rust_keyword(&ident) {
        ident.push('_');
    }
    Ok(ident)
}

fn pack_enum_doc_label(pack_id: &str) -> &'static str {
    match pack_id {
        "bootstrap" => "Bootstrap Icons",
        "carbon" => "Carbon Icons",
        "devicon" => "Devicon",
        "feather" => "Feather Icons",
        "fluentui" => "Fluent UI System Icons",
        "heroicons" => "Heroicons",
        "iconoir" => "Iconoir",
        "ionicons" => "Ionicons",
        "lobe" => "Lobe icons",
        "lucide" => "Lucide",
        "octicons" => "Octicons",
        "phosphor" => "Phosphor Icons",
        "remixicon" => "Remix Icon",
        "tabler" => "Tabler Icons",
        _ => "Icon pack",
    }
}

fn rustfmt(code: &str) -> Result<String> {
    let mut cmd = rustfmt_command()?;
    let mut child = cmd
        .args(["--emit", "stdout", "--edition", "2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Spawning rustfmt")?;

    {
        let stdin = child.stdin.as_mut().context("Opening rustfmt stdin")?;
        stdin
            .write_all(code.as_bytes())
            .context("Writing to rustfmt stdin")?;
    }

    let output = child.wait_with_output().context("Waiting on rustfmt")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("rustfmt failed: {stderr}");
    }

    String::from_utf8(output.stdout).context("Decoding rustfmt output")
}

/// F-032: prefer toolchain sysroot rustfmt; fall back to `rustup run <channel> rustfmt`.
fn rustfmt_command() -> Result<Command> {
    if let Some(bin) = rustfmt_from_sysroot()? {
        return Ok(Command::new(bin));
    }

    let channel = toolchain_channel().with_context(|| {
        "rustfmt not found under rustc sysroot bin/; also failed to read channel from rust-toolchain.toml for rustup fallback"
    })?;

    let probe = Command::new("rustup")
        .args(["run", &channel, "rustfmt", "--version"])
        .output();
    let ok = matches!(&probe, Ok(o) if o.status.success());
    if !ok {
        let detail = match probe {
            Ok(o) => String::from_utf8_lossy(&o.stderr).trim().to_owned(),
            Err(e) => e.to_string(),
        };
        bail!(
            "rustfmt is required for code generation but was not found.\n\
             Tried: `rustc --print sysroot` → <sysroot>/bin/rustfmt[.exe]\n\
             Tried: `rustup run {channel} rustfmt` ({detail})\n\
             Install with: rustup component add rustfmt --toolchain {channel}"
        );
    }

    let mut cmd = Command::new("rustup");
    cmd.args(["run", &channel, "rustfmt"]);
    Ok(cmd)
}

fn rustfmt_from_sysroot() -> Result<Option<PathBuf>> {
    let output = Command::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .context("running `rustc --print sysroot` to locate rustfmt")?;
    if !output.status.success() {
        return Ok(None);
    }
    let sysroot = String::from_utf8_lossy(&output.stdout);
    let sysroot = sysroot.trim();
    if sysroot.is_empty() {
        return Ok(None);
    }
    let bin_name = if cfg!(windows) {
        "rustfmt.exe"
    } else {
        "rustfmt"
    };
    let path = Path::new(sysroot).join("bin").join(bin_name);
    if path.is_file() {
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

fn toolchain_channel() -> Result<String> {
    let path = repo_root()?.join("rust-toolchain.toml");
    let raw = fs::read_to_string(&path).with_context(|| format!("Reading {}", path.display()))?;
    let value: toml::Value = raw
        .parse()
        .with_context(|| format!("Parsing {}", path.display()))?;
    value
        .get("toolchain")
        .and_then(|t| t.get("channel"))
        .and_then(|c| c.as_str())
        .map(str::to_owned)
        .with_context(|| format!("{} missing [toolchain].channel", path.display()))
}

fn write_output(path: &Path, content: &str, check: bool) -> Result<()> {
    match fs::read_to_string(path) {
        Ok(existing) => {
            if existing != content {
                if check {
                    bail!("Generated file differs: {}", path.display());
                }
                write_atomic(path, content)?;
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            if check {
                bail!("Generated file missing: {}", path.display());
            }
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Creating {}", parent.display()))?;
            }
            write_atomic(path, content)?;
        }
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

/// F-033: same-dir `path.tmp` then `fs::rename` for atomic replace.
///
/// Destination may already exist; `fs::rename` replaces it on supported
/// platforms (including current Windows). Do not pre-delete the target.
fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let mut tmp_os = path.as_os_str().to_owned();
    tmp_os.push(".tmp");
    let tmp_path = PathBuf::from(tmp_os);

    fs::write(&tmp_path, content)
        .with_context(|| format!("Writing temporary {}", tmp_path.display()))?;

    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Renaming {} -> {} (atomic replace)",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

fn push_line(out: &mut String, line: &str) {
    out.push_str(line);
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_icon_names() {
        assert_eq!(normalize_icon_name("arrow-left").unwrap(), "ArrowLeft");
        assert_eq!(normalize_icon_name("0-circle").unwrap(), "Icon0Circle");
        assert_eq!(normalize_icon_name("type").unwrap(), "Type_");
    }

    #[test]
    fn normalize_pack_requires_codepoints() {
        let pack = PackMap {
            pack_id: "demo".to_string(),
            source_path: PathBuf::from("demo.json"),
            variants: vec![Variant {
                id: "regular".to_string(),
                style: Style::Regular,
                size: Size::Regular,
                family: "Demo Regular".to_string(),
                ttf_asset_path: "assets/fonts/demo.ttf".to_string(),
                feature: None,
            }],
            icons: vec![Icon {
                name: "missing".to_string(),
                codepoint: None,
                overrides: BTreeMap::new(),
                availability: None,
            }],
        };

        let err = normalize_pack(pack).unwrap_err();
        assert!(err.to_string().contains("has no codepoint or overrides"));
    }

    #[test]
    fn normalize_pack_uses_overrides_when_no_default() {
        let mut overrides = BTreeMap::new();
        overrides.insert("regular".to_string(), 42);

        let pack = PackMap {
            pack_id: "demo".to_string(),
            source_path: PathBuf::from("demo.json"),
            variants: vec![Variant {
                id: "regular".to_string(),
                style: Style::Regular,
                size: Size::Regular,
                family: "Demo Regular".to_string(),
                ttf_asset_path: "assets/fonts/demo.ttf".to_string(),
                feature: None,
            }],
            icons: vec![Icon {
                name: "icon".to_string(),
                codepoint: None,
                overrides,
                availability: None,
            }],
        };

        let normalized = normalize_pack(pack).unwrap();
        assert_eq!(normalized.icons.len(), 1);
        assert_eq!(normalized.icons[0].codepoints.len(), 1);
        assert_eq!(normalized.icons[0].codepoints[0].1, 42);
    }

    #[test]
    fn size_deserializes_custom_number() {
        let raw = r#"
        {
          "pack_id": "demo",
          "variants": [
            {
              "id": "regular-20",
              "style": "Regular",
              "size": 20,
              "family": "Demo Regular",
              "ttf_asset_path": "assets/fonts/demo/demo.ttf"
            }
          ],
          "icons": [
            { "name": "demo", "codepoint": 1 }
          ]
        }"#;
        let map: PackMap = serde_json::from_str(raw).unwrap();
        assert_eq!(map.variants.len(), 1);
        assert_eq!(map.variants[0].size, Size::Custom(20));
    }

    #[test]
    fn collect_font_assets_deduplicates_by_path() {
        let pack = NormalizedPack {
            pack_id: "demo".to_string(),
            variants: vec![
                VariantInfo {
                    id: "regular".to_string(),
                    key: VariantKey {
                        style: Style::Regular,
                        size: Size::Regular,
                    },
                    family: "Demo Regular".to_string(),
                    ttf_asset_path: "assets/fonts/demo/demo.ttf".to_string(),
                    feature: None,
                },
                VariantInfo {
                    id: "filled".to_string(),
                    key: VariantKey {
                        style: Style::Filled,
                        size: Size::Regular,
                    },
                    family: "Demo Regular".to_string(),
                    ttf_asset_path: "assets/fonts/demo/demo.ttf".to_string(),
                    feature: None,
                },
            ],
            icons: Vec::new(),
        };

        let (assets, _, _) = collect_font_assets(&pack).unwrap();
        assert_eq!(assets.len(), 1);
    }

    #[test]
    fn collect_font_assets_preserves_feature_when_uniform() {
        let pack = NormalizedPack {
            pack_id: "demo".to_string(),
            variants: vec![
                VariantInfo {
                    id: "tiny".to_string(),
                    key: VariantKey {
                        style: Style::Regular,
                        size: Size::Tiny,
                    },
                    family: "Demo Tiny".to_string(),
                    ttf_asset_path: "assets/fonts/demo/demo-tiny.ttf".to_string(),
                    feature: Some("demo-tiny".to_string()),
                },
                VariantInfo {
                    id: "tiny-filled".to_string(),
                    key: VariantKey {
                        style: Style::Filled,
                        size: Size::Tiny,
                    },
                    family: "Demo Tiny".to_string(),
                    ttf_asset_path: "assets/fonts/demo/demo-tiny.ttf".to_string(),
                    feature: Some("demo-tiny".to_string()),
                },
            ],
            icons: Vec::new(),
        };

        let (assets, _, _) = collect_font_assets(&pack).unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].feature.as_deref(), Some("demo-tiny"));
    }

    #[test]
    fn pack_enum_ident_fluentui_is_fluent_ui() {
        assert_eq!(pack_enum_ident("fluentui").unwrap(), "FluentUi");
        assert_eq!(pack_enum_ident("feather").unwrap(), "Feather");
    }

    #[test]
    fn render_pack_omits_dead_icon_surface() {
        let pack = NormalizedPack {
            pack_id: "demo".to_string(),
            variants: vec![VariantInfo {
                id: "regular".to_string(),
                key: VariantKey {
                    style: Style::Regular,
                    size: Size::Regular,
                },
                family: "Demo Regular".to_string(),
                ttf_asset_path: "assets/fonts/demo/demo.ttf".to_string(),
                feature: None,
            }],
            icons: vec![NormalizedIcon {
                name: "arrow".to_string(),
                ident: "Arrow".to_string(),
                codepoints: vec![(
                    VariantKey {
                        style: Style::Regular,
                        size: Size::Regular,
                    },
                    1,
                )],
            }],
        };

        let rendered = render_pack(&pack).unwrap();
        assert!(!rendered.contains("enum Icon"));
        assert!(!rendered.contains("impl Icon"));
        assert!(!rendered.contains("FONT_ASSETS"));
        assert!(!rendered.contains("VARIANT_ASSETS"));
        assert!(!rendered.contains("allow(dead_code)"));
        assert!(!rendered.contains("enum_variant_names"));
        assert!(rendered.contains("ICON_NAMES"));
        assert!(rendered.contains("ICON_ENTRIES"));
        assert!(rendered.contains("fn icon_entry"));
        assert!(rendered.contains("mod gen_invariants"));
        assert!(rendered.contains("icon_entries_sorted_by_name"));
        assert!(rendered.contains("icon_names_match_entries"));
        assert!(rendered.contains("every_entry_name_resolves"));
        assert!(rendered.contains("name_cmp_matches_bytes_cmp"));
    }

    #[test]
    fn expected_generated_rs_names_are_mod_plus_packs() {
        let packs = [
            NormalizedPack {
                pack_id: "alpha".to_string(),
                variants: Vec::new(),
                icons: Vec::new(),
            },
            NormalizedPack {
                pack_id: "beta".to_string(),
                variants: Vec::new(),
                icons: Vec::new(),
            },
        ];
        let expected = expected_generated_rs_names(&packs);
        assert_eq!(
            expected,
            BTreeSet::from([
                "mod.rs".to_string(),
                "alpha.rs".to_string(),
                "beta.rs".to_string(),
            ])
        );
    }

    #[test]
    fn orphan_generated_rs_names_lists_unexpected_only() {
        let expected = BTreeSet::from(["mod.rs".to_string(), "demo.rs".to_string()]);
        let on_disk = vec![
            "demo.rs".to_string(),
            "stale.rs".to_string(),
            "mod.rs".to_string(),
            "extra.rs".to_string(),
        ];
        assert_eq!(
            orphan_generated_rs_names(&expected, on_disk),
            vec!["extra.rs".to_string(), "stale.rs".to_string()]
        );
    }

    #[test]
    fn render_mod_emits_cold_resolve_helpers() {
        let pack = NormalizedPack {
            pack_id: "demo".to_string(),
            variants: vec![VariantInfo {
                id: "regular".to_string(),
                key: VariantKey {
                    style: Style::Regular,
                    size: Size::Regular,
                },
                family: "Demo Regular".to_string(),
                ttf_asset_path: "assets/fonts/demo/demo.ttf".to_string(),
                feature: None,
            }],
            icons: Vec::new(),
        };
        let rendered = render_mod(&[pack]).unwrap();
        assert!(rendered.contains("#[cold]"));
        assert!(rendered.contains("fn resolve_invariant_family"));
        assert!(rendered.contains("fn resolve_invariant_codepoint"));
        assert!(rendered.contains("resolve_invariant_family(family)"));
        assert!(rendered.contains("pub fn resolve_all"));
        assert!(rendered.contains("ICON_ENTRIES.len()"));
        assert!(rendered.contains("for entry in demo::ICON_ENTRIES"));
    }

    #[test]
    fn check_pack_features_rejects_unknown_variant_feature() {
        let dir = tempfile_dir();
        let cargo = dir.join("Cargo.toml");
        fs::write(
            &cargo,
            r#"
[package]
name = "probe"
version = "0.0.0"

[features]
pack-demo = []
"#,
        )
        .unwrap();

        let packs = vec![NormalizedPack {
            pack_id: "demo".to_string(),
            variants: vec![VariantInfo {
                id: "tiny".to_string(),
                key: VariantKey {
                    style: Style::Regular,
                    size: Size::Tiny,
                },
                family: "Demo Tiny".to_string(),
                ttf_asset_path: "assets/fonts/demo/demo-tiny.ttf".to_string(),
                feature: Some("missing-feature".to_string()),
            }],
            icons: Vec::new(),
        }];

        let err = check_pack_features(&dir, &packs).unwrap_err();
        assert!(
            err.to_string()
                .contains("variant.feature 'missing-feature' not found")
        );
    }

    fn tempfile_dir() -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!(
            "iconflow-xtask-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
