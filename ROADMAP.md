# Roadmap

iconflow is a **GUI-agnostic** icon font crate: pack data and lookup only. Consumers
register fonts and draw glyphs in their own toolkit (egui, iced, or anything else).
There are **no** `egui` or `iced` Cargo features and **no** public per-pack icon enums
(`LucideIcon`, `PhosphorIcon`, etc.). The public lookup surface is `fonts` / `list` /
`try_icon` plus the `Pack` enum (including `Pack::FluentUi`).

## Current (2.0)

- 14 icon packs behind `pack-*` features (`all-packs` for demos/CI)
- Optional size gates: `heroicons-tiny`, `heroicons-mini`, `octicons-tiny`
- Committed `src/generated/**` from `cargo xtask gen`
- Examples under `examples/v2.0/` (egui + iced; toolkit is a **dev-dependency**, not a crate feature); `docs/historical/v1.0/` kept as a 1.0.0 API snapshot
- CI: fmt, clippy, tests, docs, package list, licenses, bench compile smoke

## Near term

| Focus | Notes |
|---|---|
| Pack / font updates | Refresh TTFs and maps; keep generator deterministic |
| Docs & examples | Keep install pins and toolkit snippets in sync with egui/iced releases |
| Performance | Maintain binary-search lookup benches; no public typed-icon hot path |
| Packaging | Keep `CHANGELOG.md` in the crate package; lean `include` |

## Explicit non-goals

- Bundling egui/iced as optional crate features
- Public infallible typed icon enums per pack
- An iced (or egui) **version** CI matrix inside this repo — examples track one pinned toolkit version in `Cargo.toml` `[dev-dependencies]`

## Maintenance

Bug fixes, dependency bumps, and pack refreshes land as needed. Breaking API changes follow SemVer (see `CHANGELOG.md`).
