//! Criterion harness for icon lookup (F-040 / N-G-009).
//!
//! Workload variety (perf-book benchmarking): first / mid-table / last-alphabet
//! hits per pack, plus Phosphor Thin last-name probe, miss, variant_miss, and
//! picker_frame stress.
//!
//! Packs under test: Phosphor, Tabler, FluentUi, Feather (small-pack control).
//! Variant-miss (probed once): all four packs return
//! [`IconError::VariantUnavailable`] for `(Style::Regular, Size::Tiny)` on a
//! known-existing icon name.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use iconflow::{IconError, Pack, Size, Style, list, try_icon};

const MISSING_NAME: &str = "__iconflow_missing__";

/// Hit names: first / mid-table / last Regular+Regular resolvers (probed).
const HIT_PHOSPHOR_FIRST: &str = "acorn";
const HIT_PHOSPHOR_MID: &str = "head-circuit";
const HIT_PHOSPHOR_LAST: &str = "youtube-logo";
/// Phosphor encodes Thin as a distinct last-table name (maps require `-thin`).
const HIT_PHOSPHOR_LAST_THIN: &str = "youtube-logo-thin";

const HIT_TABLER_FIRST: &str = "a-b";
const HIT_TABLER_MID: &str = "git-pull-request";
const HIT_TABLER_LAST: &str = "zzz-off";

const HIT_FLUENTUI_FIRST: &str = "access-time";
const HIT_FLUENTUI_MID: &str = "layout-column-one-third-right";
const HIT_FLUENTUI_LAST: &str = "zoom-out";

const HIT_FEATHER_FIRST: &str = "activity";
const HIT_FEATHER_MID: &str = "link-2";
const HIT_FEATHER_LAST: &str = "zoom-out";

/// Variant-miss: existing name + unavailable (Style, Size) per pack.
const VARIANT_MISS: (Style, Size) = (Style::Regular, Size::Tiny);

/// One existing name per pack (used by miss/variant_miss/picker wiring).
fn packs() -> [(Pack, &'static str); 4] {
    [
        (Pack::Phosphor, HIT_PHOSPHOR_FIRST),
        (Pack::Tabler, HIT_TABLER_FIRST),
        (Pack::FluentUi, HIT_FLUENTUI_FIRST),
        (Pack::Feather, HIT_FEATHER_FIRST),
    ]
}

/// Hit positions that distinguish binary search from early-exit linear scans.
fn hit_cases() -> [(Pack, &'static str, &'static str); 12] {
    [
        (Pack::Phosphor, "first", HIT_PHOSPHOR_FIRST),
        (Pack::Phosphor, "mid", HIT_PHOSPHOR_MID),
        (Pack::Phosphor, "last", HIT_PHOSPHOR_LAST),
        (Pack::Tabler, "first", HIT_TABLER_FIRST),
        (Pack::Tabler, "mid", HIT_TABLER_MID),
        (Pack::Tabler, "last", HIT_TABLER_LAST),
        (Pack::FluentUi, "first", HIT_FLUENTUI_FIRST),
        (Pack::FluentUi, "mid", HIT_FLUENTUI_MID),
        (Pack::FluentUi, "last", HIT_FLUENTUI_LAST),
        (Pack::Feather, "first", HIT_FEATHER_FIRST),
        (Pack::Feather, "mid", HIT_FEATHER_MID),
        (Pack::Feather, "last", HIT_FEATHER_LAST),
    ]
}

fn hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit");
    for (pack, pos, name) in hit_cases() {
        assert!(
            try_icon(pack, name, Style::Regular, Size::Regular).is_ok(),
            "hit name `{name}` ({pos}) must resolve for {pack:?}"
        );
        group.bench_function(format!("{pack:?}/{pos}"), |b| {
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

    // D-R3-09 / R3-N-08: Phosphor Thin last-name worst-case (maps use `-thin`).
    assert!(
        try_icon(
            Pack::Phosphor,
            HIT_PHOSPHOR_LAST_THIN,
            Style::Thin,
            Size::Regular
        )
        .is_ok(),
        "Thin last probe `{HIT_PHOSPHOR_LAST_THIN}` must resolve for Phosphor"
    );
    group.bench_function("Phosphor/last_thin", |b| {
        b.iter(|| {
            try_icon(
                black_box(Pack::Phosphor),
                black_box(HIT_PHOSPHOR_LAST_THIN),
                black_box(Style::Thin),
                black_box(Size::Regular),
            )
        })
    });

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
