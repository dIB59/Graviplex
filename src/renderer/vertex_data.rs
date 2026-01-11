use crate::simulation::Body;
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
pub struct Instance {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: [u8; 4],
    // Physics data (renderer ignores this but keeps stride aligned with GpuParticle)
    pub velocity: [f32; 2],
    pub mass: f32,
    pub id: u32,
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Unorm8x4];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: 32, // Stride for GpuParticle/Instance with physics data
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl From<&Body> for Instance {
    fn from(body: &Body) -> Self {
        Instance {
            position: [body.position[0] as f32, body.position[1] as f32],
            radius: body.radius as f32,
            color: body.color,
            velocity: [body.velocity[0] as f32, body.velocity[1] as f32],
            mass: body.mass as f32,
            id: body.id,
        }
    }
}
