# iconflow egui demo (1.0)

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/iconflow/main/examples/v1.0/egui_demo/egui_demo.png" alt="egui_demo" width="900" />
</p>

Historical demo matching **iconflow 1.0.0** (`Pack::Fluentui`, public `packs::*` typed API existed alongside the string path).

This tree is a reference snapshot. It is **not** registered in `Cargo.toml` on the 2.x branch — `Pack::Fluentui` and related 1.0 surface do not compile against 2.0.0.

## Run (on tag `v1.0.0`)

```bash
git checkout v1.0.0
cargo run --example egui_demo --features all-packs
```

For current crate demos, see `examples/v2.0/egui_demo/`.
