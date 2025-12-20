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
                    _ => Err(E::unknown_variant(value, &["Tiny", "Mini", "Regular", "Large"])),
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

    let mut outputs = Vec::new();
    outputs.push((
        generated_dir.join("mod.rs"),
        rustfmt(&render_mod(&normalized)?)?,
    ));

    for pack in &normalized {
        let path = generated_dir.join(format!("{}.rs", pack.pack_id));
        outputs.push((path, rustfmt(&render_pack(pack)?)?));
    }

    for (path, content) in &outputs {
        write_output(path, content, check)?;
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
        let feature_set = asset_feature_sets
            .get(&path)
            .cloned()
            .unwrap_or_default();
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
        push_line(&mut out, &format!("pub mod {pack_id};"));
        push_line(&mut out, "");
    }

    push_line(
        &mut out,
        "#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]",
    );
    push_line(&mut out, "pub enum Pack {");
    for pack in packs {
        let pack_id = &pack.pack_id;
        let ident = pack_enum_ident(pack_id)?;
        push_line(&mut out, &format!("    #[cfg(feature = \"pack-{pack_id}\")]"));
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
            push_line(&mut out, &format!("        {pack_id}::{},", asset.const_ident));
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
    push_line(&mut out, "pub fn list(pack: Pack) -> &'static [&'static str] {");
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
    push_line(&mut out, "pub fn list(_pack: Pack) -> &'static [&'static str] {");
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
        push_line(&mut out, &format!("        Pack::{ident} => resolve_icon("));
        push_line(&mut out, &format!("            {pack_id}::PACK_ID,"));
        push_line(&mut out, "            name,");
        push_line(&mut out, "            style,");
        push_line(&mut out, "            size,");
        push_line(
            &mut out,
            &format!("            {pack_id}::icon_available(name),"),
        );
        push_line(
            &mut out,
            &format!("            {pack_id}::variant_info(style, size).map(|info| info.family),"),
        );
        push_line(
            &mut out,
            &format!("            {pack_id}::icon_codepoint(name, crate::core::VariantKey {{ style, size }}),"),
        );
        push_line(&mut out, "        ),");
    }
    push_line(&mut out, "    }");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, &format!("#[cfg(not(any({any_packs_cfg})))]"));
    push_line(
        &mut out,
        "pub fn try_icon(_pack: Pack, _name: &str, _style: Style, _size: Size) -> Result<IconRef, IconError> {",
    );
    push_line(&mut out, "    Err(IconError::PackDisabled { pack: \"none\" })");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, &format!("#[cfg(any({any_packs_cfg}))]"));
    push_line(
        &mut out,
        "fn resolve_icon(pack: &'static str, name: &str, style: Style, size: Size, available: Option<&'static [(Style, Size)]>, family: Option<&'static str>, codepoint: Option<u32>) -> Result<IconRef, IconError> {",
    );
    push_line(&mut out, "    let available = match available {");
    push_line(&mut out, "        Some(available) => available,");
    push_line(
        &mut out,
        "        None => return Err(IconError::IconNotFound { pack, name: name.to_string() }),",
    );
    push_line(&mut out, "    };");
    push_line(&mut out, "");
    push_line(&mut out, "    if !available.contains(&(style, size)) {");
    push_line(
        &mut out,
        "        return Err(IconError::VariantUnavailable { pack, name: name.to_string(), requested: (style, size), available });",
    );
    push_line(&mut out, "    }");
    push_line(&mut out, "");
    push_line(
        &mut out,
        "    let family = family.expect(\"Icon variant should have a font family\");",
    );
    push_line(
        &mut out,
        "    let codepoint = codepoint.expect(\"Icon variant should have a codepoint\");",
    );
    push_line(&mut out, "    Ok(IconRef { family, codepoint })");
    push_line(&mut out, "}");

    Ok(out)
}

fn render_pack(pack: &NormalizedPack) -> Result<String> {
    let mut out = String::new();
    push_line(&mut out, "// @generated by xtask gen. DO NOT EDIT.");
    push_line(
        &mut out,
        "use crate::core::{FontAsset, IconRef, Size, Style, VariantKey};",
    );
    push_line(&mut out, "");
    push_line(
        &mut out,
        &format!("pub const PACK_ID: &str = \"{}\";", pack.pack_id),
    );
    push_line(&mut out, "");

    let (assets, asset_const_by_path, variant_feature_by_key) =
        collect_font_assets(pack)?;

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
    push_line(&mut out, "pub const FONT_ASSETS: &[FontAsset] = &[");
    for asset in &assets {
        if let Some(feature) = &asset.feature {
            push_line(&mut out, &cfg_attr_line(feature, 4));
        }
        push_line(&mut out, &format!("    {},", asset.const_ident));
    }
    push_line(&mut out, "];");
    push_line(&mut out, "");

    push_line(
        &mut out,
        "pub const VARIANT_ASSETS: &[(VariantKey, FontAsset)] = &[",
    );
    for variant in &pack.variants {
        if let Some(feature) = &variant.feature {
            push_line(&mut out, &cfg_attr_line(feature, 4));
        }
        let const_ident = asset_const_by_path
            .get(&variant.ttf_asset_path.replace('\\', "/"))
            .ok_or_else(|| anyhow::anyhow!("Missing asset const for {}", variant.ttf_asset_path))?;
        push_line(
            &mut out,
            &format!(
                "    ({}, {}),",
                variant_key_expr(variant.key),
                const_ident
            ),
        );
    }
    push_line(&mut out, "];");
    push_line(&mut out, "");

    push_line(
        &mut out,
        "#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]",
    );
    push_line(&mut out, "pub enum Icon {");
    for icon in &pack.icons {
        push_line(&mut out, &format!("    {},", icon.ident));
    }
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, "impl Icon {");
    push_line(&mut out, "    pub fn name(self) -> &'static str {");
    push_line(&mut out, "        match self {");
    for icon in &pack.icons {
        push_line(
            &mut out,
            &format!("            Icon::{} => \"{}\",", icon.ident, icon.name),
        );
    }
    push_line(&mut out, "        }");
    push_line(&mut out, "    }");
    push_line(&mut out, "");
    push_line(
        &mut out,
        "    pub fn icon(self, style: Style, size: Size) -> IconRef {",
    );
    push_line(
        &mut out,
        "        let name = self.name();",
    );
    push_line(&mut out, "        let available = icon_available(name).unwrap_or(&[]);");
    push_line(&mut out, "        if !available.contains(&(style, size)) {");
    push_line(
        &mut out,
        "            panic!(\"Icon '{}' is not available in {:?}/{:?}. Available: {:?}\", name, style, size, available);",
    );
    push_line(&mut out, "        }");
    push_line(
        &mut out,
        "        let variant = variant_info(style, size).unwrap_or_else(|| {",
    );
    push_line(
        &mut out,
        "            panic!(\"Variant {:?}/{:?} is not available for pack {}\", style, size, PACK_ID)",
    );
    push_line(&mut out, "        });");
    push_line(
        &mut out,
        "        let codepoint = icon_codepoint(name, variant.key).unwrap_or_else(|| {",
    );
    push_line(
        &mut out,
        "            panic!(\"Icon '{}' is not available in {:?}/{:?}\", name, style, size)",
    );
    push_line(&mut out, "        });");
    push_line(
        &mut out,
        "        IconRef { family: variant.family, codepoint }",
    );
    push_line(&mut out, "    }");
    push_line(&mut out, "}");
    push_line(&mut out, "");

    push_line(&mut out, "pub const ICON_NAMES: &[&str] = &[");
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
                &format!("    (Style::{}, {}),", key.style.as_rust(), key.size.rust_expr()),
            );
        }
        push_line(&mut out, "];");
        push_line(&mut out, "");
    }

    push_line(&mut out, "#[derive(Clone, Copy, Debug)]");
    push_line(&mut out, "pub(crate) struct IconCodepoints {");
    push_line(&mut out, "    pub name: &'static str,");
    push_line(
        &mut out,
        "    pub codepoints: &'static [(VariantKey, u32)],",
    );
    push_line(&mut out, "}");
    push_line(&mut out, "");
    push_line(
        &mut out,
        "pub(crate) const ICON_CODEPOINTS: &[IconCodepoints] = &[",
    );
    for icon in &pack.icons {
        let const_name = icon_codepoints_const_ident(&icon.ident)?;
        push_line(
            &mut out,
            &format!(
                "    IconCodepoints {{ name: \"{}\", codepoints: {} }},",
                icon.name, const_name
            ),
        );
    }
    push_line(&mut out, "];");
    push_line(&mut out, "");

    push_line(&mut out, "#[derive(Clone, Copy, Debug)]");
    push_line(&mut out, "pub(crate) struct IconAvailability {");
    push_line(&mut out, "    pub name: &'static str,");
    push_line(&mut out, "    pub available: &'static [(Style, Size)],");
    push_line(&mut out, "}");
    push_line(&mut out, "");
    push_line(
        &mut out,
        "pub(crate) const ICON_AVAILABILITY: &[IconAvailability] = &[",
    );
    for icon in &pack.icons {
        let const_name = icon_available_const_ident(&icon.ident)?;
        push_line(
            &mut out,
            &format!(
                "    IconAvailability {{ name: \"{}\", available: {} }},",
                icon.name, const_name
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
        "pub(crate) fn icon_codepoint(name: &str, key: VariantKey) -> Option<u32> {",
    );
    push_line(
        &mut out,
        "    ICON_CODEPOINTS.iter().find(|entry| entry.name == name).and_then(|entry| {",
    );
    push_line(
        &mut out,
        "        entry.codepoints.iter().find(|(k, _)| *k == key).map(|(_, cp)| *cp)",
    );
    push_line(&mut out, "    })");
    push_line(&mut out, "}");
    push_line(&mut out, "");
    push_line(
        &mut out,
        "pub(crate) fn icon_available(name: &str) -> Option<&'static [(Style, Size)]> {",
    );
    push_line(
        &mut out,
        "    ICON_AVAILABILITY.iter().find(|entry| entry.name == name).map(|entry| entry.available)",
    );
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
    format!("{:indent$}#[cfg(feature = \"{feature}\")]", "", indent = indent)
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
    let mut ident = to_pascal_case(pack_id)?;
    if is_rust_keyword(&ident) {
        ident.push('_');
    }
    Ok(ident)
}

fn rustfmt(code: &str) -> Result<String> {
    let mut child = Command::new("rustfmt")
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

fn write_output(path: &Path, content: &str, check: bool) -> Result<()> {
    match fs::read_to_string(path) {
        Ok(existing) => {
            if existing != content {
                if check {
                    bail!("Generated file differs: {}", path.display());
                }
                fs::write(path, content).with_context(|| format!("Writing {}", path.display()))?;
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
            fs::write(path, content).with_context(|| format!("Writing {}", path.display()))?;
        }
        Err(err) => return Err(err.into()),
    }
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
}
