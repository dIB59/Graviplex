use crate::core::geometry::Circle;
use bytemuck::NoUninit;

#[repr(C)]
#[derive(Clone, Copy, NoUninit, Debug)]
pub struct Vertex {
    pub pos: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, NoUninit, Debug)]
pub struct CircleInstance {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: [f32; 4],
}

impl CircleInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array!(1 => Float32x2, 2 => Float32, 3 => Float32x4);

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CircleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl From<Circle> for CircleInstance {
    fn from(c: Circle) -> Self {
        Self {
            position: c.center.into(),
            radius: c.radius,
            color: c.color.into(),
        }
    }
}

impl From<&Circle> for CircleInstance {
    fn from(c: &Circle) -> Self {
        Self {
            position: c.center.into(),
            radius: c.radius,
            color: c.color.into(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, NoUninit, Debug)]
pub struct PhysicsInstance {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: [u8; 4],
    // Physics data (renderer ignores this but keeps stride aligned with GpuParticle)
    pub velocity: [f32; 2],
    pub mass: f32,
    pub id: u32,
}

impl PhysicsInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array!(1 => Float32x2, 2 => Float32, 3 => Unorm8x4);

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: 32, // Stride for GpuParticle/PhysicsInstance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl From<Circle> for PhysicsInstance {
    fn from(c: Circle) -> Self {
        Self {
            position: c.center.into(),
            radius: c.radius,
            color: [
                (c.color.r * 255.0) as u8,
                (c.color.g * 255.0) as u8,
                (c.color.b * 255.0) as u8,
                (c.color.a * 255.0) as u8,
            ],
            velocity: [0.0, 0.0],
            mass: 1.0,
            id: 0,
        }
    }
}
