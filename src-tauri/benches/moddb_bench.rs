/// Criterion benchmarks for the ModDB layer rebuild operations.
///
/// These measure the hot-path cost of re-seeding modifier layers when the user
/// changes their passive tree, equips an item, or switches class.
///
/// Run with:
///   cargo bench --bench moddb_bench
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rustc_hash::FxHashMap;
use rusty_builds_lib::{
    data::{Class, GameData},
    item::types::{Item, ItemSlot},
    modifier::ModDBLayers,
};
use std::path::PathBuf;
use std::sync::OnceLock;

fn resource_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Loads GameData once for all benchmarks to avoid I/O bottlenecks
fn game_data() -> &'static GameData {
    static GD: OnceLock<GameData> = OnceLock::new();
    GD.get_or_init(|| GameData::load_from_dir(resource_dir()).expect("bench data load failed"))
}

// ---------------------------------------------------------------------------
// Benchmark: rebuild_tree at different node counts
// ---------------------------------------------------------------------------
fn bench_rebuild_tree(c: &mut Criterion) {
    let game_data = game_data();
    let class = Class::Marauder(None);

    // Pre-collect node ID sets at different sizes.
    let all_ids: Vec<u32> = game_data
        .repoe_tree
        .passives
        .keys()
        .filter(|&&id| {
            game_data
                .repoe_tree
                .get_passive(id)
                .map(|p| p.ascendancy.is_none() && !p.stats.is_empty())
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    let mut group = c.benchmark_group("moddb/rebuild_tree");
    for &n in &[50usize, 100, 300, 600] {
        let ids: Vec<u32> = all_ids.iter().take(n).cloned().collect();
        let mut layers = ModDBLayers::default();
        layers.rebuild_class(&class, &game_data.tree);

        group.bench_with_input(BenchmarkId::from_parameter(n), &ids, |b, node_ids| {
            b.iter(|| layers.rebuild_tree(black_box(node_ids), black_box(game_data)))
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: rebuild_class
// ---------------------------------------------------------------------------
fn bench_rebuild_class(c: &mut Criterion) {
    let game_data = game_data();
    let mut layers = ModDBLayers::default();

    c.bench_function("moddb/rebuild_class", |b| {
        b.iter(|| {
            layers.rebuild_class(
                black_box(&Class::Marauder(None)),
                black_box(&game_data.tree),
            )
        })
    });
}

// ---------------------------------------------------------------------------
// Benchmark: rebuild_items — empty equipped map (baseline overhead)
// ---------------------------------------------------------------------------
fn bench_rebuild_items_empty(c: &mut Criterion) {
    let equipped: FxHashMap<ItemSlot, Item> = FxHashMap::default();
    let mut layers = ModDBLayers::default();

    c.bench_function("moddb/rebuild_items_empty", |b| {
        b.iter(|| layers.rebuild_items(black_box(&equipped)))
    });
}

// ---------------------------------------------------------------------------
// Benchmark: layers.merged() — how long does flattening all layers take?
// ---------------------------------------------------------------------------
fn bench_moddb_merge(c: &mut Criterion) {
    let game_data = game_data();
    let class = Class::Marauder(None);
    let ids: Vec<u32> = game_data
        .repoe_tree
        .passives
        .keys()
        .filter(|&&id| {
            game_data
                .repoe_tree
                .get_passive(id)
                .map(|p| p.ascendancy.is_none() && !p.stats.is_empty())
                .unwrap_or(false)
        })
        .take(100)
        .cloned()
        .collect();
    let mut layers = ModDBLayers::default();
    layers.rebuild_class(&class, &game_data.tree);
    layers.rebuild_tree(&ids, &game_data);

    c.bench_function("moddb/merge_100_nodes", |b| {
        b.iter(|| black_box(layers.merged()))
    });
}

criterion_group!(
    benches,
    bench_rebuild_tree,
    bench_rebuild_class,
    bench_rebuild_items_empty,
    bench_moddb_merge,
);
criterion_main!(benches);
