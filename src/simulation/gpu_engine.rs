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
    pub seed: u32,
    pub interaction_pos: [f32; 2],
    pub interaction_radius: f32,
    pub interaction_strength: f32,
    pub _padding: [u32; 57], // Pad to 256 bytes (64 * 4 bytes)
}

pub struct GpuEngine {
    pub particle_buffer: Buffer,
    pub params_buffer: Buffer,

    pub resolve_collisions_pipeline: ComputePipeline,
    pub init_pipeline: ComputePipeline,
    pub update_pipeline: ComputePipeline,

    bind_group: BindGroup,
    num_particles: u32,
    device: Device,
}

impl GpuEngine {
    pub fn new(device: &Device, _queue: &Queue, num_particles: u32) -> Self {
        let particle_size = (num_particles as usize * std::mem::size_of::<GpuParticle>()) as u64;

        let particle_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particles"),
            size: particle_size,
            usage: BufferUsages::STORAGE
                | BufferUsages::VERTEX
                | BufferUsages::COPY_DST
                | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Simulation Params"),
            size: (std::mem::size_of::<GpuParams>() * 256) as u64, // Space for 256 aligned chunks
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(include_wgsl!("../shaders/nbody.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Simulation Bind Group Layout"),
            entries: &[
                // 0: Particles
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
                // 1: Params
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(
                            core::num::NonZeroU64::new(std::mem::size_of::<GpuParams>() as u64)
                                .unwrap(),
                        ),
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
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &params_buffer,
                        offset: 0,
                        size: Some(
                            core::num::NonZeroU64::new(std::mem::size_of::<GpuParams>() as u64)
                                .unwrap(),
                        ),
                    }),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Simulation Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let resolve_collisions_pipeline =
            device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("Resolve Collisions Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("resolve_collisions"),
                cache: None,
                compilation_options: Default::default(),
            });

        let init_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Simulation Init Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("init_particles"),
            cache: None,
            compilation_options: Default::default(),
        });

        let update_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Simulation Update Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("update_gravity"),
            cache: None,
            compilation_options: Default::default(),
        });

        Self {
            particle_buffer,
            params_buffer,
            resolve_collisions_pipeline,
            init_pipeline,
            update_pipeline,
            bind_group,
            num_particles,
            device: device.clone(),
        }
    }

    pub fn init(&self, queue: &Queue) {
        let params = GpuParams {
            dt: 0.0,
            gravity: 0.0,
            num_particles: self.num_particles,
            seed: rand::random(),
            interaction_pos: [0.0, 0.0],
            interaction_radius: 0.0,
            interaction_strength: 0.0,
            _padding: [0; 57],
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Init Encoder"),
            });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Init Compute Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.init_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[0]);
            let workgroups = (self.num_particles + 255) / 256;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }
        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn update(
        &self,
        device: &Device,
        queue: &Queue,
        dt: f32,
        gravity: f32,
        _theta: f32,
        interaction: Option<([f32; 2], f32, f32)>,
    ) {
        let (i_pos, i_rad, i_str) = interaction.unwrap_or(([0.0, 0.0], 0.0, 0.0));

        // Only need one set of params for brute force
        let params = GpuParams {
            dt,
            gravity,
            num_particles: self.num_particles,
            seed: 0,
            interaction_pos: i_pos,
            interaction_radius: i_rad,
            interaction_strength: i_str,
            _padding: [0; 57],
        };

        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Simulation Encoder"),
        });

        // 1. Physics Pass (Brute Force)
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Physics Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.update_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[0]);
            let workgroups = (self.num_particles + 255) / 256;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }

        // 2. Collision Pass (Brute Force)
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Collision Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.resolve_collisions_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[0]);
            let workgroups = (self.num_particles + 255) / 256;
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
