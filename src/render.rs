use ::ultraviolet::Vec2;
use bytemuck::{NoUninit, Pod, Zeroable};
use rand::Rng;

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
pub struct Instance {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: [u8; 4],
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Unorm8x4];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();
        Instance {
            position: [rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0)],
            radius: rng.gen_range(0.01..0.05),
            color: [
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                rng.gen_range(0..=255),
                255, // Fully opaque
            ],
        }
    }

    pub fn default() -> Self {
        let rng = rand::rng();
        Instance {
            position: [0.0, 0.0],
            radius: 0.1,
            color: [1, 1, 1, 1],
        }
    }
}
