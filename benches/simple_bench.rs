use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use graviplex::simulation::{
    BarnesHutGravityStrategy, CollisionStrategyEnum, GravityStrategyEnum, KdTreeCollision,
    NaiveCollisionStrategy, NaiveGravityStrategy, NoCollisionStrategy, QuadtreeCollision,
    Simulation,
};

fn bench_simulation_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation_update");

    // Benchmark with 10000 particles and default settings
    group.bench_function("10000_particles_default", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    group.finish();
}

fn bench_simulation_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("gravity_strategies");

    // Barnes-Hut (default)
    group.bench_function("barnes_hut_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_gravity_strategy(GravityStrategyEnum::BarnesHut(
            BarnesHutGravityStrategy::new(0.5, 0.01f64),
        ));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    // Naive (for comparison)
    group.bench_function("naive_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_gravity_strategy(GravityStrategyEnum::Naive(NaiveGravityStrategy));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    group.finish();
}

fn bench_collision_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("collision_strategies");

    // Quadtree (default)
    group.bench_function("quadtree_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_collision_strategy(CollisionStrategyEnum::Quadtree(QuadtreeCollision::new()));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    // KD-Tree
    group.bench_function("kd_tree_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_collision_strategy(CollisionStrategyEnum::KdTree(KdTreeCollision::new()));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    // Naive collision
    group.bench_function("naive_collision_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_collision_strategy(CollisionStrategyEnum::Naive(NaiveCollisionStrategy));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    // No collision
    group.bench_function("no_collision_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_collision_strategy(CollisionStrategyEnum::None(NoCollisionStrategy));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    group.finish();
}

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("particle_scaling");

    // Test different particle counts
    for particle_count in [1000, 5000, 10000, 25000, 50000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(particle_count),
            particle_count,
            |b, &count| {
                let mut sim = Simulation::new();
                sim.generate_bodies(count);

                b.iter(|| {
                    sim.update(black_box(0.01f32));
                });
            },
        );
    }

    group.finish();
}

fn bench_quadtree_only(c: &mut Criterion) {
    use graviplex::simulation::spatial::{Quad, Quadtree};

    let mut group = c.benchmark_group("quadtree_operations");

    group.bench_function("build_10000", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);
        let px = sim.state.px.clone();
        let py = sim.state.py.clone();
        let masses = sim.state.masses.clone();

        b.iter(|| {
            let mut quadtree = Quadtree::new(0.5, 0.01f64);
            let root_quad = Quad::new_containing(&px, &py);
            quadtree.build(&px, &py, &masses, root_quad);

            black_box(&quadtree);
        });
    });

    group.bench_function("search_radius_10000", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);
        let px = sim.state.px.clone();
        let py = sim.state.py.clone();
        let masses = sim.state.masses.clone();
        let radii = sim.state.radii.clone();

        let mut quadtree = Quadtree::new(0.5, 0.01f64);
        let root_quad = Quad::new_containing(&px, &py);
        quadtree.build(&px, &py, &masses, root_quad);

        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..px.len() {
                let pos = [px[i], py[i]];
                quadtree.search_radius(pos, radii[i] * 5.0, &mut results);
                black_box(&results);
                results.clear();
            }
        });
    });

    group.finish();
}
fn bench_kdtree_only(c: &mut Criterion) {
    use graviplex::simulation::spatial::KdTree;

    let mut group = c.benchmark_group("kdtree_operations");

    group.bench_function("build_10000", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);
        let px = sim.state.px.clone();
        let py = sim.state.py.clone();

        b.iter(|| {
            let mut points: Vec<(usize, [f64; 2])> =
                (0..px.len()).map(|i| (i, [px[i], py[i]])).collect();
            let mut tree = KdTree::new();
            tree.build(&mut points);
            black_box(tree);
        });
    });

    group.bench_function("query_10000", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);
        let px = sim.state.px.clone();
        let py = sim.state.py.clone();
        let radii = sim.state.radii.clone();

        // Pre-build tree
        let mut points: Vec<(usize, [f64; 2])> =
            (0..px.len()).map(|i| (i, [px[i], py[i]])).collect();
        let mut tree = KdTree::new();
        tree.build(&mut points);

        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..px.len() {
                let pos = [px[i], py[i]];
                tree.search_radius(pos, radii[i] * 5.0, &mut results);
                black_box(&results);
                results.clear();
            }
        });
    });

    group.finish();
}

fn bench_body_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("body_generation");

    group.bench_function("generate_10000", |b| {
        b.iter(|| {
            let mut sim = Simulation::new();
            sim.generate_bodies(black_box(10000));
            black_box(&sim);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simulation_update,
    bench_simulation_strategies,
    bench_collision_strategies,
    bench_scaling,
    bench_quadtree_only,
    bench_kdtree_only,
    bench_body_generation,
);

criterion_main!(benches);
