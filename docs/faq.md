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
iconflow = { version = "2.0", features = ["pack-bootstrap"] }
```

## Unknown icon name or variant

`try_icon` is fallible. An unknown name yields `IconError::IconNotFound`; a known name with
an unsupported `(style, size)` yields `IconError::VariantUnavailable` (with `available`).
Neither case panics. The `name` field on those variants is `Cow<'static, str>`.

## Fluent UI pack variant

The pack enum variant is `Pack::FluentUi` (feature `pack-fluentui`). There is no
`Pack::Fluentui` spelling.
