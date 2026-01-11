use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use graviplex::simulation::{
    BarnesHutGravityStrategy, KdTreeCollision, NaiveCollisionStrategy, NaiveGravityStrategy,
    NoCollisionStrategy, Simulation,
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
        sim.set_gravity_strategy(Box::new(BarnesHutGravityStrategy::new(0.5, 0.01f64)));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    // Naive (for comparison)
    group.bench_function("naive_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_gravity_strategy(Box::new(NaiveGravityStrategy));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    group.finish();
}

fn bench_collision_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("collision_strategies");

    // KD-Tree (default)
    group.bench_function("kd_tree_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_collision_strategy(Box::new(KdTreeCollision));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    // Naive collision
    group.bench_function("naive_collision_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_collision_strategy(Box::new(NaiveCollisionStrategy));
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01f32));
        });
    });

    // No collision
    group.bench_function("no_collision_10000", |b| {
        let mut sim = Simulation::new();
        sim.set_collision_strategy(Box::new(NoCollisionStrategy));
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
        let bodies: Vec<_> = sim.bodies().to_vec();

        b.iter(|| {
            let mut quadtree = Quadtree::new(0.5, 0.01f64);
            let positions: Vec<[f64; 2]> = bodies.iter().map(|b| b.position).collect();
            let root_quad = Quad::new_containing(&positions);
            quadtree.clear(root_quad);

            for body in &bodies {
                quadtree.insert(body.position, body.mass);
            }

            quadtree.propagate();

            black_box(&quadtree);
        });
    });

    group.bench_function("query_10000", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);
        let bodies: Vec<_> = sim.bodies().to_vec();

        // Pre-build tree
        let mut quadtree = Quadtree::new(0.5, 0.01f64);
        let positions: Vec<[f64; 2]> = bodies.iter().map(|b| b.position).collect();
        let root_quad = Quad::new_containing(&positions);
        quadtree.clear(root_quad);

        for body in &bodies {
            quadtree.insert(body.position, body.mass);
        }
        quadtree.propagate();

        b.iter(|| {
            for body in &bodies {
                let acc = quadtree.acc(body.position, 100.0);
                black_box(acc);
            }
        });
    });

    group.finish();
}

fn bench_kdtree_only(c: &mut Criterion) {
    use graviplex::simulation::spatial::{build_kd_tree, search_radius};

    let mut group = c.benchmark_group("kdtree_operations");

    group.bench_function("build_10000", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);
        let bodies: Vec<_> = sim.bodies().to_vec();

        b.iter(|| {
            let mut points: Vec<(usize, [f64; 2])> = bodies
                .iter()
                .enumerate()
                .map(|(idx, body)| (idx, body.position))
                .collect();

            let tree = build_kd_tree(&mut points, 0);
            black_box(tree);
        });
    });

    group.bench_function("query_10000", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);
        let bodies: Vec<_> = sim.bodies().to_vec();

        // Pre-build tree
        let mut points: Vec<(usize, [f64; 2])> = bodies
            .iter()
            .enumerate()
            .map(|(idx, body)| (idx, body.position))
            .collect();
        let tree = build_kd_tree(&mut points, 0);

        b.iter(|| {
            let mut results = Vec::new();
            for body in &bodies {
                search_radius(&tree, body.position, body.radius * 5.0, 0, &mut results);
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
