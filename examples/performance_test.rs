//! Performance test rendering 1 million particles using GPU buffers.

use graviplex::prelude::*;
use graviplex::advanced::CircleInstance;

struct PerfTestGame {
    particle_buffer: Option<wgpu::Buffer>,
}

impl GameLoop for PerfTestGame {
    fn init(&mut self, _world: &mut World, gfx: &Graphics) {
        let count = 1_000_000;
        let mut instances = Vec::with_capacity(count);

        for _ in 0..count {
            let x = (rand::random::<f32>() - 0.5) * 2000.0;
            let y = (rand::random::<f32>() - 0.5) * 2000.0;
            let r = rand::random::<f32>() * 5.0 + 1.0;
            instances.push(CircleInstance::new(
                [x, y],
                r,
                [rand::random(), rand::random(), rand::random(), 1.0],
            ));
        }

        let buffer = gfx.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("1M Particle Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        });
        self.particle_buffer = Some(buffer);
    }

    fn update(&mut self, _world: &mut World, _res: &Resources) {}

    fn render(&mut self, _world: &World, draw: &mut DrawContext) {
        if let Some(buffer) = &self.particle_buffer {
            draw.circles_from_buffer(buffer, 1_000_000);
        }
    }
}

fn main() {
    App::build(PerfTestGame {
        particle_buffer: None,
    })
    .title("Graviplex Performance Test - 1M Particles")
    .size(1200, 800)
    .run()
    .unwrap();
}
