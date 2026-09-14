//! Criterion harness for icon lookup (F-040 / D1–D2).
//!
//! Packs under test: Phosphor, Tabler, Fluentui.
//! Variant-miss combos (probed once): all three packs return
//! [`IconError::VariantUnavailable`] for `(Style::Regular, Size::Tiny)` on a
//! known-existing icon name.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use iconflow::{list, try_icon, IconError, Pack, Size, Style};

const MISSING_NAME: &str = "__iconflow_missing__";

/// Hit names: first `list(pack)` entry that succeeds with Regular/Regular.
const HIT_PHOSPHOR: &str = "acorn";
const HIT_TABLER: &str = "a-b";
const HIT_FLUENTUI: &str = "access-time";

/// Variant-miss: existing name + unavailable (Style, Size) per pack.
const VARIANT_MISS: (Style, Size) = (Style::Regular, Size::Tiny);

fn packs() -> [(Pack, &'static str); 3] {
    [
        (Pack::Phosphor, HIT_PHOSPHOR),
        (Pack::Tabler, HIT_TABLER),
        (Pack::Fluentui, HIT_FLUENTUI),
    ]
}

fn hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit");
    for (pack, name) in packs() {
        debug_assert!(
            try_icon(pack, name, Style::Regular, Size::Regular).is_ok(),
            "hit name `{name}` must resolve for {pack:?}"
        );
        group.bench_function(format!("{pack:?}"), |b| {
            b.iter(|| {
                try_icon(
                    black_box(pack),
                    black_box(name),
                    black_box(Style::Regular),
                    black_box(Size::Regular),
                )
            })
        });
    }
    group.finish();
}

fn miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("miss");
    for (pack, _) in packs() {
        group.bench_function(format!("{pack:?}"), |b| {
            b.iter(|| {
                try_icon(
                    black_box(pack),
                    black_box(MISSING_NAME),
                    black_box(Style::Regular),
                    black_box(Size::Regular),
                )
            })
        });
    }
    group.finish();
}

fn variant_miss(c: &mut Criterion) {
    let (style, size) = VARIANT_MISS;
    let mut group = c.benchmark_group("variant_miss");
    for (pack, name) in packs() {
        match try_icon(pack, name, style, size) {
            Err(IconError::VariantUnavailable { .. }) => {}
            other => panic!(
                "expected VariantUnavailable for {pack:?} `{name}` {style:?}/{size:?}, got {other:?}"
            ),
        }
        group.bench_function(format!("{pack:?}"), |b| {
            b.iter(|| {
                try_icon(
                    black_box(pack),
                    black_box(name),
                    black_box(style),
                    black_box(size),
                )
            })
        });
    }
    group.finish();
}

fn picker_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("picker_frame");
    for (pack, _) in packs() {
        group.bench_function(format!("{pack:?}"), |b| {
            b.iter(|| {
                for name in list(black_box(pack)) {
                    let _ = try_icon(
                        black_box(pack),
                        black_box(*name),
                        black_box(Style::Regular),
                        black_box(Size::Regular),
                    );
                }
            })
        });
    }
    group.finish();
}

criterion_group!(benches, hit, miss, variant_miss, picker_frame);
criterion_main!(benches);
