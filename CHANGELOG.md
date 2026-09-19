# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- README: pack table sorted by font weight (icons + MiB + feature); trademark note for brand glyphs (Devicon / Lobe)
- Docs: `IconError::PackDisabled` documented as unused on normal `Pack` paths (kept for stable `#[non_exhaustive]` surface)
- Docs sync for 2.1: ROADMAP current section, FAQ/quickstart anchors, crates.io README quickstart link, Cargo.toml pack count

### Removed

- CI `licenses` job (`cargo-about` HTML drift compare)
- Internal handoff docs `docs/spec.md`, `docs/plan.md`
- Unused `about.toml` / `about.hbs` (cargo-about inputs after licenses CI removal)

## [2.1.0] - 2026-09-15

### Added

- `resolve_all(pack, style, size)` — linear full-pack resolve for picker grids (order matches `list`); Criterion `picker_frame` now measures cold `resolve_all` + warm dense `Vec` (not `HashMap`)

### Changed

- Docs/benches recommend cold `resolve_all` + warm dense `Vec` for picker grids (avoid consumer `HashMap<&str, IconRef>` memo) — methodology / recommended path note, not a crate bug fix

## [2.0.0] - 2026-09-14

### Added

- Criterion benches (`benches/lookup`) for phosphor/tabler/fluentui/feather hit/miss/variant-miss + picker-frame
- CI jobs: rustdoc `-D warnings`, MSRV 1.92.0 check, `cargo package --list` + publish dry-run, cargo-about drift gate, bench `--no-run`
- `[lints.clippy]` + `clippy.toml` disallowed-types policy
- Workspace `[profile.release]` for example/bench release builds
- `[profile.bench]` with `strip = "none"` and `panic = "unwind"`
- Common traits on public types (`Display` / `Default` / `Hash` / `Ord` as applicable per type)
- `CHANGELOG.md` included in the published crate package (`Cargo.toml` `include`)

### Changed

- **Breaking:** unknown icon name or unavailable `(style, size)` variant returns `Err(IconError::…)` instead of panicking
- **Breaking:** `Pack::Fluentui` renamed to `Pack::FluentUi`
- **Breaking:** `IconError` is `#[non_exhaustive]` (match arms must allow future variants)
- **Breaking:** `IconError` name fields use `Cow<'static, str>` instead of `String`
- **Breaking:** pack modules and generated pack internals are `pub(crate)`; curated public surface is crate-root only (`fonts` / `list` / `try_icon` / `Pack` / core types)
- **Breaking:** `core` is `pub(crate)` (no longer a documented public module); `VariantKey` is crate-private
- Icon name lookup uses `binary_search` over merged `IconEntry` tables
- Package `include` no longer ships `/assets/maps/*.json` or `/assets/schema/*.json` (runtime uses embedded generated data)

### Removed

- **Breaking:** typed per-pack `Icon` API (`enum Icon`, `Icon::name` / `Icon::icon`, `FONT_ASSETS`, `VARIANT_ASSETS`, and `iconflow::packs::*` re-exports). Use `try_icon` / `list` / `fonts` instead
- **Breaking:** no-op Cargo features `heroicons-regular` and `octicons-regular` (and their `all-packs` entries). Regular assets remain available via `pack-heroicons` / `pack-octicons`; keep `heroicons-tiny` / `heroicons-mini` / `octicons-tiny` for real size gates
- Dead `feather_svg` tree / package surface

### Fixed

- Docs: install pins and quickstart/FAQ snippets aligned with the public API; buildable demos under `examples/v2.0/` (1.0 snapshots retained under `docs/historical/v1.0/`)
- Phosphor: icon names with IcoMoon aliases were mangled by comma-glue — `selection.json` stores aliases comma-separated (`"folder-open, folder-notch-open"`) and the loader normalized the raw string, registering 108 names (18 base icons × 6 styles) under glued names like `folder-open-folder-notch-open`; they now resolve under their canonical names (`folder-open`, `pulse`, …) and the mangled names no longer resolve (#2 by @NicolasDrapier)

## [1.0.0] - 2025-12-21

### Added

- First public release of iconflow: feature-gated embedded icon packs (Bootstrap, Carbon, Devicon, Feather, Fluent UI, Heroicons, Iconoir, Ionicons, Lobe, Lucide, Octicons, Phosphor, Remix Icon, Tabler)
- String lookup API: `fonts()`, `list(pack)`, `try_icon(pack, name, style, size)`
- Public types: `Pack`, `FontAsset`, `IconRef`, `Size`, `Style`, `IconError`
- Per-pack typed `Icon` enums and font tables via `iconflow::packs::*` (alongside the string API)
- Size-variant Cargo features for Heroicons (`tiny` / `mini` / `regular`) and Octicons (`tiny` / `regular`)
- egui and iced demo examples
- MIT license, font third-party notices, and crates.io packaging metadata (MSRV 1.92, edition 2024)
