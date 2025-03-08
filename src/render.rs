use bytemuck::{Pod, Zeroable};
use ::ultraviolet::Vec2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct View {
    pub(crate) position: Vec2,
    pub(crate) scale: f32,
    pub(crate) x: u16,
    pub(crate) y: u16,
}

unsafe impl Zeroable for View {}
unsafe impl Pod for View {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: Vec2,
    pub color: [u8; 4],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Unorm8x4];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Instance {
    pub position: Vec2,
    pub radius: f32,
    pub color: [u8; 4],
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32, 2 => Unorm8x4];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
