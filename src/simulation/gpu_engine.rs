use bytemuck::{Pod, Zeroable};
use wgpu::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuParticle {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: u32,
    pub velocity: [f32; 2],
    pub mass: f32,
    pub id: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuParams {
    pub dt: f32,
    pub gravity: f32,
    pub num_particles: u32,
    pub theta: f32,
}

pub struct GpuEngine {
    pub particle_buffer: Buffer,
    params_buffer: Buffer,
    compute_pipeline: ComputePipeline,
    bind_group: BindGroup,
    num_particles: u32,
}

impl GpuEngine {
    pub fn new(device: &Device, queue: &Queue, initial_particles: &[GpuParticle]) -> Self {
        let num_particles = initial_particles.len() as u32;

        let particle_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Simulation Particles"),
            size: (initial_particles.len() * std::mem::size_of::<GpuParticle>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        queue.write_buffer(&particle_buffer, 0, bytemuck::cast_slice(initial_particles));

        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Simulation Params"),
            size: std::mem::size_of::<GpuParams>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(include_wgsl!("../shaders/nbody.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Simulation Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Simulation Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Simulation Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Simulation Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("update_gravity"),
            cache: None,
            compilation_options: Default::default(),
        });

        Self {
            particle_buffer,
            params_buffer,
            compute_pipeline,
            bind_group,
            num_particles,
        }
    }

    pub fn update(&self, device: &Device, queue: &Queue, dt: f32, gravity: f32) {
        let params = GpuParams {
            dt,
            gravity,
            num_particles: self.num_particles,
            theta: 0.5,
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Simulation Encoder"),
        });

        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Simulation Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            let workgroups = (self.num_particles + 255) / 256;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
