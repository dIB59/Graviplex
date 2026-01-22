use graviplex::core::color::Color;
use graviplex::*;
use wgpu::util::DeviceExt;
use wgpu::*;

struct TestGame {
    use_raw: bool,
    particle_buffer: Option<Buffer>,
}

impl GameLoop for TestGame {
    fn init(&mut self, gpu: &GpuContext) {
        if self.use_raw {
            let instances = vec![
                CircleInstance {
                    position: [0.0, 0.0],
                    radius: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                };
                1_000_000
            ];

            let buffer = gpu
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Test Particle Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: BufferUsages::VERTEX | BufferUsages::STORAGE,
                });
            self.particle_buffer = Some(buffer);
        }
    }

    fn update(&mut self, _dt: f32, _gpu: &GpuContext) {}

    fn render(&mut self, draw: &mut DrawContext) {
        if self.use_raw {
            if let Some(buffer) = &self.particle_buffer {
                draw.draw_circles_raw(buffer, 1_000_000);
            }
        } else {
            // Test batching using rich domain models
            draw.draw_circle(Circle::new(Vec2::new(0.0, 0.0), 10.0, Color::RED));
            draw.draw_circle(Circle::new(Vec2::new(100.0, 100.0), 20.0, Color::GREEN));
            draw.draw_line([0.0, 0.0], [100.0, 100.0], [0.0, 0.0, 1.0, 1.0]);
        }
    }
}

#[cfg(test)]
mod e2e_tests {
    use super::*;

    #[test]
    fn test_e2e_headless_rendering() {
        let gpu = GpuContext::new();
        let camera = Camera2D::new([0.0, 0.0], 1.0, [1024.0, 1024.0]);
        let format = TextureFormat::Rgba8UnormSrgb;

        let mut circle_pipeline = CirclePipeline::new(&gpu.device, format, &camera);
        let mut line_pipeline = LinePipeline::new(
            "Test",
            include_wgsl!("../src/shaders/line_shader.wgsl"),
            &gpu.device,
            format,
            &camera,
        );

        let texture = gpu.device.create_texture(&TextureDescriptor {
            label: Some("Dummy Texture"),
            size: Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());

        let mut game = TestGame {
            use_raw: false,
            particle_buffer: None,
        };
        game.init(&gpu);

        let mut draw = DrawContext {
            gpu: &gpu,
            view: &view,
            camera: &camera,
            circle_pipeline: &mut circle_pipeline,
            line_pipeline: &mut line_pipeline,
        };

        // Render standard batching
        game.render(&mut draw);
        assert_eq!(draw.circle_pipeline.staging_count(), 2);
        assert_eq!(draw.line_pipeline.staging_count(), 1);

        draw.flush();
        assert_eq!(circle_pipeline.staging_count(), 0);
        assert_eq!(line_pipeline.staging_count(), 0);

        // Test 1M performance path (raw)
        let mut perf_game = TestGame {
            use_raw: true,
            particle_buffer: None,
        };
        perf_game.init(&gpu);

        {
            let mut draw_perf = DrawContext {
                gpu: &gpu,
                view: &view,
                camera: &camera,
                circle_pipeline: &mut circle_pipeline,
                line_pipeline: &mut line_pipeline,
            };
            perf_game.render(&mut draw_perf);
            // raw rendering doesn't use staging_instances
            assert_eq!(draw_perf.circle_pipeline.staging_count(), 0);
        }
    }
}
