# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Fixed

## [1.0.0] - 2026-09-14

### Added

- Criterion benches (`benches/lookup`) for phosphor/tabler/fluentui hit/miss/variant-miss + picker-frame
- CI jobs: rustdoc `-D warnings`, MSRV 1.92.0 check, package list + publish dry-run, cargo-about drift gate, bench `--no-run`
- `[lints.clippy]` + `clippy.toml` disallowed-types policy
- Workspace `[profile.release]` for example/bench release builds

### Changed

- Icon name lookup uses `binary_search` / merged `IconEntry` tables
- `Icon::icon` is fallible (`Result`)
- `IconError.name` is `Cow<'static, str>`
- Pack modules are `pub(crate)`; curated public surface via crate root
- Dead `feather_svg` removed from tree / package surface

### Fixed

- Docs: install pins `version = "1.0"`; quickstart core snippet compilable; example README paths under `examples/v1.0/`
