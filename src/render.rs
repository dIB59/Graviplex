use std::fmt;

use bytemuck::{NoUninit, Pod, Zeroable};
use rand::Rng;
use ultraviolet::Vec2;

use crate::simulation::Body;

#[repr(C)]
#[derive(Clone, Copy, NoUninit, Debug)]
pub struct Vertex {
    pub pos: [f32; 2],
}

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

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Instance {{ position: ({:.2}, {:.2}), radius: {:.2}, color: rgba({}, {}, {}, {}) }}",
            self.position[0],
            self.position[1],
            self.radius,
            self.color[0],
            self.color[1],
            self.color[2],
            self.color[3],
        )
    }
}

impl From<&Body> for Instance {
    fn from(value: &Body) -> Self {
        return Instance {
            color: value.color,
            position: value.position,
            radius: value.radius,
        };
    }
}

impl Default for Instance {
    fn default() -> Self {
        Instance {
            position: [0.0, 0.0],
            radius: 0.1,
            color: [1, 1, 1, 1],
        }
    }
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
}
