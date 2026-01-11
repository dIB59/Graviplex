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
pub struct GpuNode {
    pub children: [i32; 4],
    pub pos: [f32; 2],
    pub mass: f32,
    pub parent: i32,
    pub size: f32,
    pub next: i32,
    pub _padding: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuParams {
    pub dt: f32,
    pub gravity: f32,
    pub num_particles: u32,
    pub theta: f32,
    pub seed: u32,
    pub pass_index: u32,
    pub num_nodes: u32,
    pub interaction_pos: [f32; 2],
    pub interaction_radius: f32,
    pub interaction_strength: f32,
    pub _padding: [u32; 53], // Pad to 256 bytes
}

pub struct GpuEngine {
    pub particle_buffer: Buffer,
    pub node_buffer: Buffer,
    pub sort_data_buffer: Buffer,
    pub sort_temp_buffer: Buffer,
    pub histogram_buffer: Buffer,
    pub atomic_counters_buffer: Buffer,
    pub params_buffer: Buffer,

    pub morton_pipeline: ComputePipeline,
    pub radix_histogram_pipeline: ComputePipeline,
    pub radix_shuffle_pipeline: ComputePipeline,
    pub tree_build_pipeline: ComputePipeline,
    pub compute_mass_pipeline: ComputePipeline,
    pub resolve_collisions_pipeline: ComputePipeline,
    pub init_pipeline: ComputePipeline,
    pub update_pipeline: ComputePipeline,
    pub bitonic_sort_pipeline: ComputePipeline,

    bind_group: BindGroup,
    num_particles: u32,
    device: Device,
}

impl GpuEngine {
    pub fn new(device: &Device, _queue: &Queue, num_particles: u32) -> Self {
        let particle_size = (num_particles as usize * std::mem::size_of::<GpuParticle>()) as u64;
        let node_size = (num_particles as usize * 2 * std::mem::size_of::<GpuNode>()) as u64;

        let particle_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Particles"),
            size: particle_size,
            usage: BufferUsages::STORAGE
                | BufferUsages::VERTEX
                | BufferUsages::COPY_DST
                | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let node_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Quadtree Nodes"),
            size: node_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sort_data_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Sort Data A"),
            size: (num_particles as usize * 8) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let sort_temp_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Sort Data B"),
            size: (num_particles as usize * 8) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let histogram_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Histograms"),
            size: (16 * 4) as u64, // 16 digits * 4 bytes
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let atomic_counters_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Atomic Counters"),
            size: (num_particles * 2 * 4) as u64, // 2N * u32
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
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
                // 2: Nodes
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 3: Sort Data (Buffer A)
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 4: Sort Temp (Buffer B)
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 5: Histograms
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 6: Atomic Counters
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
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
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &params_buffer,
                        offset: 0,
                        size: Some(
                            core::num::NonZeroU64::new(std::mem::size_of::<GpuParams>() as u64)
                                .unwrap(),
                        ),
                    }),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: node_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: sort_data_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: sort_temp_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: histogram_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 6,
                    resource: atomic_counters_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Simulation Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let morton_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Morton Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("morton_encode"),
            cache: None,
            compilation_options: Default::default(),
        });

        let radix_histogram_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Histogram Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("radix_histogram"),
            cache: None,
            compilation_options: Default::default(),
        });

        let radix_shuffle_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Shuffle Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("radix_shuffle"),
            cache: None,
            compilation_options: Default::default(),
        });

        let tree_build_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Tree Build Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("build_tree"),
            cache: None,
            compilation_options: Default::default(),
        });

        let compute_mass_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Mass Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("compute_mass"),
            cache: None,
            compilation_options: Default::default(),
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

        let bitonic_sort_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Bitonic Sort Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("bitonic_sort_step"),
            cache: None,
            compilation_options: Default::default(),
        });

        Self {
            particle_buffer,
            node_buffer,
            sort_data_buffer,
            sort_temp_buffer,
            histogram_buffer,
            atomic_counters_buffer,
            params_buffer,
            morton_pipeline,
            radix_histogram_pipeline,
            radix_shuffle_pipeline,
            tree_build_pipeline,
            compute_mass_pipeline,
            resolve_collisions_pipeline,
            init_pipeline,
            update_pipeline,
            bitonic_sort_pipeline,
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
            theta: 0.5,
            seed: rand::random(),
            pass_index: 0,
            num_nodes: 0,
            interaction_pos: [0.0, 0.0],
            interaction_radius: 0.0,
            interaction_strength: 0.0,
            _padding: [0; 53],
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
        theta: f32,
        interaction: Option<([f32; 2], f32, f32)>,
    ) {
        let (i_pos, i_rad, i_str) = interaction.unwrap_or(([0.0, 0.0], 0.0, 0.0));

        // Only need one set of params for brute force
        let params = GpuParams {
            dt,
            gravity,
            num_particles: self.num_particles,
            theta,
            seed: 0,
            pass_index: 0,
            num_nodes: 0,
            interaction_pos: i_pos,
            interaction_radius: i_rad,
            interaction_strength: i_str,
            _padding: [0; 53],
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
