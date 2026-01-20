use bytemuck::{Pod, Zeroable};
use wgpu::*;

// ============================================================================
// GPU Data Structures
// ============================================================================

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
    pub _padding: [u32; 60], // Pad to 256 bytes (64 * 4 bytes)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MortonEntry {
    pub key: u32,
    pub particle_idx: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TreeNode {
    pub first_child: u32,
    pub particle_start: u32,
    pub particle_count: u32,
    pub level: u32,
    pub center_of_mass: [f32; 2],
    pub total_mass: f32,
    pub _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Bounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct TreeParams {
    pub num_particles: u32,
    pub num_nodes: u32,
    pub theta_sq: f32,
    pub epsilon_sq: f32,
    pub gravity: f32,
    pub dt: f32,
    pub max_level: u32,
    pub _padding: u32,
}

// ============================================================================
// GPU Engine
// ============================================================================

pub struct GpuEngine {
    // Particle data
    pub particle_buffer: Buffer,
    pub params_buffer: Buffer,

    // Morton tree data
    pub morton_buffer: Buffer,
    pub morton_temp_buffer: Buffer,
    pub tree_buffer: Buffer,
    pub bounds_buffer: Buffer,
    pub tree_params_buffer: Buffer,
    pub histogram_buffer: Buffer,

    // Original pipelines (for init and collisions)
    pub resolve_collisions_pipeline: ComputePipeline,
    pub init_pipeline: ComputePipeline,

    // Morton tree pipelines
    pub compute_morton_pipeline: ComputePipeline,
    pub build_tree_pipeline: ComputePipeline,
    pub compute_com_pipeline: ComputePipeline,
    pub barnes_hut_pipeline: ComputePipeline,
    pub integrate_pipeline: ComputePipeline,

    // Bind groups
    bind_group: BindGroup,
    tree_bind_group: BindGroup,
    sort_bind_group: BindGroup,

    num_particles: u32,
    max_nodes: u32,
    device: Device,

    // Feature flag
    use_morton_tree: bool,
}

impl GpuEngine {
    pub fn new(device: &Device, _queue: &Queue, num_particles: u32) -> Self {
        let particle_size = (num_particles as usize * std::mem::size_of::<GpuParticle>()) as u64;
        let morton_size = (num_particles as usize * std::mem::size_of::<MortonEntry>()) as u64;

        // Estimate max nodes: roughly 4/3 * num_particles for a balanced tree
        let max_nodes = ((num_particles as f32 * 1.5) as u32).max(1024);
        let tree_size = (max_nodes as usize * std::mem::size_of::<TreeNode>()) as u64;

        // Histogram for radix sort: 16 buckets per workgroup
        let num_workgroups = (num_particles + 255) / 256;
        let histogram_size = (num_workgroups as usize * 16 * std::mem::size_of::<u32>()) as u64;

        // ====================================================================
        // Create Buffers
        // ====================================================================

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
            size: (std::mem::size_of::<GpuParams>() * 256) as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let morton_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Morton Entries"),
            size: morton_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let morton_temp_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Morton Temp"),
            size: morton_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let tree_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Tree Nodes"),
            size: tree_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bounds_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Bounds"),
            size: std::mem::size_of::<Bounds>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let tree_params_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Tree Params"),
            size: std::mem::size_of::<TreeParams>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let histogram_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Sort Histograms"),
            size: histogram_size.max(256), // Ensure minimum size
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ====================================================================
        // Load Shaders
        // ====================================================================

        let nbody_shader = device.create_shader_module(include_wgsl!("../shaders/nbody.wgsl"));
        let morton_shader =
            device.create_shader_module(include_wgsl!("../shaders/morton_tree.wgsl"));

        // ====================================================================
        // Bind Group Layouts
        // ====================================================================

        // Original layout for init/collisions
        let nbody_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("N-Body Bind Group Layout"),
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

        // Morton tree main layout: particles, morton, tree, bounds, params
        let tree_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Tree Bind Group Layout"),
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
                // 1: Morton entries
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 2: Tree nodes
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
                // 3: Bounds
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
                // 4: Tree params
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            core::num::NonZeroU64::new(std::mem::size_of::<TreeParams>() as u64)
                                .unwrap(),
                        ),
                    },
                    count: None,
                },
            ],
        });

        // Sort auxiliary layout: morton_temp, histograms
        let sort_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Sort Bind Group Layout"),
            entries: &[
                // 0: Morton temp
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
                // 1: Histograms
                BindGroupLayoutEntry {
                    binding: 1,
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

        // ====================================================================
        // Create Bind Groups
        // ====================================================================

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("N-Body Bind Group"),
            layout: &nbody_bind_group_layout,
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

        let tree_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Tree Bind Group"),
            layout: &tree_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: morton_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: tree_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: bounds_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: tree_params_buffer.as_entire_binding(),
                },
            ],
        });

        let sort_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Sort Bind Group"),
            layout: &sort_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: morton_temp_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: histogram_buffer.as_entire_binding(),
                },
            ],
        });

        // ====================================================================
        // Create Pipelines
        // ====================================================================

        let nbody_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("N-Body Pipeline Layout"),
            bind_group_layouts: &[&nbody_bind_group_layout],
            push_constant_ranges: &[],
        });

        let tree_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Tree Pipeline Layout"),
            bind_group_layouts: &[&tree_bind_group_layout, &sort_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Original N-body pipelines
        let resolve_collisions_pipeline =
            device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("Resolve Collisions Pipeline"),
                layout: Some(&nbody_pipeline_layout),
                module: &nbody_shader,
                entry_point: Some("resolve_collisions"),
                cache: None,
                compilation_options: Default::default(),
            });

        let init_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Simulation Init Pipeline"),
            layout: Some(&nbody_pipeline_layout),
            module: &nbody_shader,
            entry_point: Some("init_particles"),
            cache: None,
            compilation_options: Default::default(),
        });

        // Morton tree pipelines
        let compute_morton_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Morton Pipeline"),
            layout: Some(&tree_pipeline_layout),
            module: &morton_shader,
            entry_point: Some("compute_morton"),
            cache: None,
            compilation_options: Default::default(),
        });

        let build_tree_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Build Tree Pipeline"),
            layout: Some(&tree_pipeline_layout),
            module: &morton_shader,
            entry_point: Some("build_tree"),
            cache: None,
            compilation_options: Default::default(),
        });

        let compute_com_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute CoM Pipeline"),
            layout: Some(&tree_pipeline_layout),
            module: &morton_shader,
            entry_point: Some("compute_center_of_mass"),
            cache: None,
            compilation_options: Default::default(),
        });

        let barnes_hut_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Barnes-Hut Pipeline"),
            layout: Some(&tree_pipeline_layout),
            module: &morton_shader,
            entry_point: Some("barnes_hut_gravity"),
            cache: None,
            compilation_options: Default::default(),
        });

        let integrate_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Integrate Pipeline"),
            layout: Some(&tree_pipeline_layout),
            module: &morton_shader,
            entry_point: Some("integrate"),
            cache: None,
            compilation_options: Default::default(),
        });

        Self {
            particle_buffer,
            params_buffer,
            morton_buffer,
            morton_temp_buffer,
            tree_buffer,
            bounds_buffer,
            tree_params_buffer,
            histogram_buffer,
            resolve_collisions_pipeline,
            init_pipeline,
            compute_morton_pipeline,
            build_tree_pipeline,
            compute_com_pipeline,
            barnes_hut_pipeline,
            integrate_pipeline,
            bind_group,
            tree_bind_group,
            sort_bind_group,
            num_particles,
            max_nodes,
            device: device.clone(),
            use_morton_tree: true, // Enable Morton tree by default
        }
    }

    pub fn set_use_morton_tree(&mut self, use_tree: bool) {
        self.use_morton_tree = use_tree;
    }

    pub fn init(&self, queue: &Queue) {
        let params = GpuParams {
            dt: 0.0,
            gravity: 0.0,
            num_particles: self.num_particles,
            seed: rand::random(),
            _padding: [0; 60],
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        // Initialize bounds with large values
        let initial_bounds = Bounds {
            min_x: -100000.0,
            min_y: -100000.0,
            max_x: 100000.0,
            max_y: 100000.0,
        };
        queue.write_buffer(&self.bounds_buffer, 0, bytemuck::bytes_of(&initial_bounds));

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

    pub fn update(&self, device: &Device, queue: &Queue, dt: f32, gravity: f32, theta: f32) {
        if self.use_morton_tree {
            self.update_with_morton_tree(device, queue, dt, gravity, theta);
        } else {
            self.update_brute_force(device, queue, dt, gravity);
        }
    }

    fn update_brute_force(&self, device: &Device, queue: &Queue, dt: f32, gravity: f32) {
        let params = GpuParams {
            dt,
            gravity,
            num_particles: self.num_particles,
            seed: 0,
            _padding: [0; 60],
        };
        queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Brute Force Encoder"),
        });

        // Use original update_gravity from nbody.wgsl (need to add pipeline for this)
        // For now, skip the physics pass when using brute force mode
        // The user should define which mode they want

        // Collision Pass
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

    fn update_with_morton_tree(
        &self,
        device: &Device,
        queue: &Queue,
        dt: f32,
        gravity: f32,
        theta: f32,
    ) {
        // Update tree params
        let tree_params = TreeParams {
            num_particles: self.num_particles,
            num_nodes: self.max_nodes,
            theta_sq: theta * theta,
            epsilon_sq: 1.0, // Softening factor
            gravity,
            dt,
            max_level: 16, // Max tree depth
            _padding: 0,
        };
        queue.write_buffer(
            &self.tree_params_buffer,
            0,
            bytemuck::bytes_of(&tree_params),
        );

        let workgroups = (self.num_particles + 255) / 256;
        let node_workgroups = (self.max_nodes + 255) / 256;

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Morton Tree Encoder"),
        });

        // Pass 1: Compute Morton codes
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute Morton Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_morton_pipeline);
            cpass.set_bind_group(0, &self.tree_bind_group, &[]);
            cpass.set_bind_group(1, &self.sort_bind_group, &[]);
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Note: For a complete implementation, we would need to:
        // 1. Run radix sort passes here (multiple passes)
        // 2. Build tree hierarchy from sorted Morton codes
        // For now, we skip sorting and build a simple tree

        // Pass 2: Build tree (simplified - treats all particles as one node initially)
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Build Tree Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.build_tree_pipeline);
            cpass.set_bind_group(0, &self.tree_bind_group, &[]);
            cpass.set_bind_group(1, &self.sort_bind_group, &[]);
            cpass.dispatch_workgroups(node_workgroups, 1, 1);
        }

        // Pass 3: Compute centers of mass
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Compute CoM Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_com_pipeline);
            cpass.set_bind_group(0, &self.tree_bind_group, &[]);
            cpass.set_bind_group(1, &self.sort_bind_group, &[]);
            cpass.dispatch_workgroups(node_workgroups, 1, 1);
        }

        // Pass 4: Barnes-Hut gravity calculation
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Barnes-Hut Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.barnes_hut_pipeline);
            cpass.set_bind_group(0, &self.tree_bind_group, &[]);
            cpass.set_bind_group(1, &self.sort_bind_group, &[]);
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Pass 5: Integrate (update positions)
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Integrate Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.integrate_pipeline);
            cpass.set_bind_group(0, &self.tree_bind_group, &[]);
            cpass.set_bind_group(1, &self.sort_bind_group, &[]);
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }

        // Pass 6: Collisions (still brute force for now)
        {
            let params = GpuParams {
                dt,
                gravity,
                num_particles: self.num_particles,
                seed: 0,
                _padding: [0; 60],
            };
            queue.write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
        }

        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Collision Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.resolve_collisions_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[0]);
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
