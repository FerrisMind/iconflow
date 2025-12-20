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
iconflow = { version = "0.1", features = ["pack-bootstrap"] }
```
