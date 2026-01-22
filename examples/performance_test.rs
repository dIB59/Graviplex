use graviplex::*;
use wgpu::util::DeviceExt;

struct PerfTestGame {
    particle_buffer: Option<wgpu::Buffer>,
}

impl GameLoop for PerfTestGame {
    fn init(&mut self, gpu: &GpuContext) {
        let count = 1_000_000;
        let mut instances = Vec::with_capacity(count);

        for _ in 0..count {
            let x = (rand::random::<f32>() - 0.5) * 2000.0;
            let y = (rand::random::<f32>() - 0.5) * 2000.0;
            let r = rand::random::<f32>() * 5.0 + 1.0;
            instances.push(CircleInstance {
                position: [x, y],
                radius: r,
                color: [rand::random(), rand::random(), rand::random(), 1.0],
            });
        }

        let buffer = gpu
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("1M Particle Buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            });
        self.particle_buffer = Some(buffer);
    }

    fn update(&mut self, _dt: f32, _gpu: &GpuContext) {}

    fn render(&mut self, draw: &mut DrawContext) {
        if let Some(buffer) = &self.particle_buffer {
            draw.draw_circles_raw(buffer, 1_000_000);
        }
    }
}

fn main() {
    let game = PerfTestGame {
        particle_buffer: None,
    };
    let app = App::new(game);
    app.run().unwrap();
}
