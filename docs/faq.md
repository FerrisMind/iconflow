# FAQ

## Missing glyph (shows tofu or empty box)

Make sure you loaded the font bytes from `iconflow::fonts()` into your GUI framework before
rendering the icon. In iced 0.14, this means calling `font::load` for each `FontAsset` and
waiting for the tasks to complete.

## Wrong font family (icon shows as a letter instead)

Use the `IconRef.family` value as the font family. It is the font family stored inside the
TTF, not the file name. Passing the file name will not match in iced/egui.

## Pack feature disabled

If `Pack::Bootstrap` (or another pack) is missing, enable the feature in `Cargo.toml`:

```toml
iconflow = { version = "2.1", features = ["pack-bootstrap"] }
```

### Zero pack features vs a disabled pack (R3-N-06)

With **no** `pack-*` features enabled, `Pack` is an empty enum: there are no variants to
construct, so `list` / `try_icon` are not callable and fail at **compile time** (they do
**not** return `IconError::PackDisabled`).

When at least one pack feature is enabled, a pack you did not enable simply has no
`Pack::…` variant (again a compile error if you name it).

`IconError::PackDisabled` is kept for a stable `#[non_exhaustive]` error surface. The only
in-tree constructor is the zero-feature stub (`pack: "none"`), which you cannot hit through
a constructible `Pack`. Prefer enabling the right `pack-*` feature; do not write application
logic that expects `PackDisabled` for “forgot a feature”.

## Unknown icon name or variant

`try_icon` is fallible. An unknown name yields `IconError::IconNotFound`; a known name with
an unsupported `(style, size)` yields `IconError::VariantUnavailable` (with `available`).
Neither case panics. The `name` field on those variants is `Cow<'static, str>`.

## Default `(Style::Regular, Size::Regular)` is not universal (R3-N-07)

Do not assume every pack ships a Regular/Regular glyph. At minimum:

| Pack | Feature | Typical styles at `Size::Regular` |
|------|---------|-----------------------------------|
| Heroicons | `pack-heroicons` | `Filled`, `Outline` (not `Style::Regular`) |
| Remix Icon | `pack-remixicon` | `Filled`, `Outline` (not `Style::Regular`) |

A request for `(Style::Regular, Size::Regular)` on those packs returns
`IconError::VariantUnavailable`. Inspect `available` on that error (or the pack’s variant
tables) and pick a listed `(style, size)` — no runtime default remapping is applied.

## Fluent UI pack variant

The pack enum variant is `Pack::FluentUi` (feature `pack-fluentui`). There is no
`Pack::Fluentui` spelling.

## Icon pickers (cold + warm frames)

For a full-pack grid, prefer [`resolve_all`](https://docs.rs/iconflow/*/iconflow/fn.resolve_all.html)
once (linear table walk; order matches `list`) instead of `n × try_icon` or a
`HashMap<&str, IconRef>` memo. Keep the `Vec` and index it on warm frames — name-keyed
hash maps are slower and are not part of the crate’s hot path.

## Known limitations

Dual static name storage (`ICON_NAMES` plus each `ICON_ENTRIES[].name`) is intentional so
`list` can return `&'static [&str]` while the entry table keeps per-icon metadata.
