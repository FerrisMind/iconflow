# iconflow iced demo (1.0)

<p align="center">
  <img src="https://raw.githubusercontent.com/FerrisMind/iconflow/main/examples/v1.0/iced_demo/iced_demo.png" alt="iced_demo" width="900" />
</p>

Historical demo matching **iconflow 1.0.0** (`Pack::Fluentui`, public `packs::*` typed API existed alongside the string path).

This tree is a reference snapshot. It is **not** registered in `Cargo.toml` on the 2.x branch — `Pack::Fluentui` and related 1.0 surface do not compile against 2.0.0.

## Run (on tag `v1.0.0`)

```bash
git checkout v1.0.0
cargo run --example iced_demo --features all-packs
```

For current crate demos, see `examples/v2.0/iced_demo/`.
