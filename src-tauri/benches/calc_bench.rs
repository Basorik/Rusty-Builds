/// Criterion benchmarks for the full calculation pipeline.
///
/// Run with:
///   cargo bench --bench calc_bench
///   cargo bench --bench calc_bench -- --output-format bencher   # for CI
///
/// HTML reports are written to target/criterion/  
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rustc_hash::FxHashMap;
use rusty_builds_lib::{
    calc::calculate,
    data::{
        skills::{GemInstance, GemRef, SkillGroup},
        Class, GameData,
    },
    item::types::{Item, ItemSlot},
    modifier::ModDBLayers,
};
use std::path::PathBuf;
use std::sync::OnceLock;

fn resource_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Loads GameData once for all benchmarks to avoid I/O bottlenecks during bench suite startup
fn game_data() -> &'static GameData {
    static GD: OnceLock<GameData> = OnceLock::new();
    GD.get_or_init(|| GameData::load_from_dir(resource_dir()).expect("bench data load failed"))
}

/// Build a ModDBLayers with class + N tree nodes seeded.
fn setup_layers(game_data: &GameData, class: &Class, n_nodes: usize) -> ModDBLayers {
    let mut layers = ModDBLayers::default();
    layers.rebuild_class(class, &game_data.tree);
    let node_ids: Vec<u32> = game_data
        .repoe_tree
        .passives
        .keys()
        .filter(|&&id| {
            // Only normal/notable nodes — skip keystones and ascendancy nodes
            if let Some(p) = game_data.repoe_tree.get_passive(id) {
                p.ascendancy.is_none() && !p.stats.is_empty()
            } else {
                false
            }
        })
        .take(n_nodes)
        .cloned()
        .collect();
    layers.rebuild_tree(&node_ids, game_data);
    layers
}

/// Find the first active (non-support) skill gem id in the loaded data.
fn first_active_gem_id(game_data: &GameData) -> Option<String> {
    game_data
        .gems
        .iter()
        .find(|(_, g)| g.active_skill.is_some())
        .map(|(id, _)| id.clone())
}

// ---------------------------------------------------------------------------
// Benchmark: full calculate() — no active gem
// ---------------------------------------------------------------------------
fn bench_full_calc_no_gem(c: &mut Criterion) {
    let game_data = game_data();
    let class = Class::Marauder(None);
    let equipped: FxHashMap<ItemSlot, Item> = FxHashMap::default();

    let mut group = c.benchmark_group("full_calc");
    for n_nodes in [0usize, 50, 100, 300] {
        let layers = setup_layers(&game_data, &class, n_nodes);
        group.bench_with_input(BenchmarkId::new("no_gem", n_nodes), &n_nodes, |b, _| {
            b.iter(|| {
                black_box(calculate(
                    black_box(&layers),
                    black_box(90),
                    black_box(&class),
                    black_box(None),
                    black_box(&[]),
                    black_box(&equipped),
                    black_box(game_data),
                ))
            })
        });
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: full calculate() — with active gem (offense pipeline included)
// ---------------------------------------------------------------------------
fn bench_full_calc_with_gem(c: &mut Criterion) {
    let game_data = game_data();
    let class = Class::Marauder(None);
    let equipped: FxHashMap<ItemSlot, Item> = FxHashMap::default();
    let layers = setup_layers(&game_data, &class, 100);

    let Some(gem_id) = first_active_gem_id(&game_data) else {
        eprintln!("No active gem found in game data, skipping bench_full_calc_with_gem");
        return;
    };

    let skill_groups = vec![SkillGroup {
        id: 0,
        label: "Bench Group".into(),
        enabled: true,
        gems: vec![GemInstance {
            gem_id: gem_id.clone(),
            name: gem_id.clone(),
            is_support: false,
            level: 20,
            quality: 20,
            enabled: true,
            always_active: false,
        }],
    }];
    let gem_ref = GemRef {
        group_id: 0,
        gem_index: 0,
    };

    c.bench_function("full_calc/with_gem_100_nodes", |b| {
        b.iter(|| {
            black_box(calculate(
                black_box(&layers),
                black_box(90),
                black_box(&class),
                black_box(Some(&gem_ref)),
                black_box(&skill_groups),
                black_box(&equipped),
                black_box(game_data),
            ))
        })
    });
}

criterion_group!(benches, bench_full_calc_no_gem, bench_full_calc_with_gem,);
criterion_main!(benches);
