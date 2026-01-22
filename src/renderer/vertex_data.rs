use crate::core::geometry::Circle;
use bytemuck::NoUninit;

/// Basic vertex with 2D position.
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

/// GPU-compatible circle instance for batched rendering.
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

    /// Create a new circle instance.
    pub fn new(position: [f32; 2], radius: f32, color: [f32; 4]) -> Self {
        Self {
            position,
            radius,
            color,
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
