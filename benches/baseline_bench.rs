use criterion::{black_box, criterion_group, criterion_main, Criterion};
use graviplex_sim::nbody::Simulation;

fn bench_simulation_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("Default Simpulation");

    // Benchmark with 10000 particles and default settings
    group.bench_function("10000_particles_default", |b| {
        let mut sim = Simulation::new();
        sim.generate_bodies(10000);

        b.iter(|| {
            sim.update(black_box(0.01));
        });
    });

    group.finish();
}

criterion_group!(baseline_benches, bench_simulation_update);
criterion_main!(baseline_benches);
