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

// =============================================================================
// SPRITE INSTANCE
// =============================================================================

/// GPU-compatible sprite instance for batched textured quad rendering.
///
/// Used with texture atlases for efficient sprite batching.
#[repr(C)]
#[derive(Copy, Clone, NoUninit, Debug)]
pub struct SpriteInstance {
    /// World position (center of sprite).
    pub position: [f32; 2],
    /// Size in world units (width, height).
    pub size: [f32; 2],
    /// UV coordinates in atlas (u_min, v_min, u_max, v_max).
    pub uv_rect: [f32; 4],
    /// Color tint (multiplied with texture color).
    pub tint: [f32; 4],
    /// Rotation in radians.
    pub rotation: f32,
    /// Z-order for sorting (not sent to shader, used for CPU sorting).
    pub z_order: i32,
}

impl SpriteInstance {
    // Attributes start at location 1 (after vertex position at 0)
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array!(
        1 => Float32x2,  // position
        2 => Float32x2,  // size
        3 => Float32x4,  // uv_rect
        4 => Float32x4,  // tint
        5 => Float32     // rotation
    );

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            // Only the GPU-sent portion (exclude z_order)
            array_stride: std::mem::size_of::<SpriteInstanceGpu>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    /// Create a new sprite instance.
    pub fn new(
        position: [f32; 2],
        size: [f32; 2],
        uv_rect: [f32; 4],
        tint: [f32; 4],
        rotation: f32,
        z_order: i32,
    ) -> Self {
        Self {
            position,
            size,
            uv_rect,
            tint,
            rotation,
            z_order,
        }
    }

    /// Convert to GPU-compatible version (without z_order).
    pub fn to_gpu(&self) -> SpriteInstanceGpu {
        SpriteInstanceGpu {
            position: self.position,
            size: self.size,
            uv_rect: self.uv_rect,
            tint: self.tint,
            rotation: self.rotation,
        }
    }
}

/// GPU-only sprite instance data (excludes z_order used for CPU sorting).
#[repr(C)]
#[derive(Copy, Clone, NoUninit, Debug)]
pub struct SpriteInstanceGpu {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub uv_rect: [f32; 4],
    pub tint: [f32; 4],
    pub rotation: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // UNIT TESTS - Vertex
    // =========================================================================

    #[test]
    fn test_vertex_creation() {
        let v = Vertex { pos: [1.0, 2.0] };
        assert_eq!(v.pos, [1.0, 2.0]);
    }

    #[test]
    fn test_vertex_copy_clone() {
        let v1 = Vertex { pos: [3.0, 4.0] };
        let v2 = v1; // Copy
        let v3 = v1.clone();

        assert_eq!(v1.pos, v2.pos);
        assert_eq!(v1.pos, v3.pos);
    }

    #[test]
    fn test_vertex_desc_attributes() {
        let desc = Vertex::desc();

        assert_eq!(desc.array_stride, std::mem::size_of::<Vertex>() as u64);
        assert_eq!(desc.step_mode, wgpu::VertexStepMode::Vertex);
        assert_eq!(desc.attributes.len(), 1);
    }

    // =========================================================================
    // UNIT TESTS - CircleInstance
    // =========================================================================

    #[test]
    fn test_circle_instance_new() {
        let ci = CircleInstance::new([10.0, 20.0], 5.0, [1.0, 0.0, 0.0, 1.0]);

        assert_eq!(ci.position, [10.0, 20.0]);
        assert_eq!(ci.radius, 5.0);
        assert_eq!(ci.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_circle_instance_from_circle() {
        use crate::core::color::Color;
        use crate::core::geometry::Circle;
        use crate::core::math::Vec2;

        let circle = Circle::new(Vec2::new(100.0, 200.0), 50.0, Color::BLUE);
        let instance: CircleInstance = circle.into();

        assert_eq!(instance.position, [100.0, 200.0]);
        assert_eq!(instance.radius, 50.0);
    }

    #[test]
    fn test_circle_instance_from_circle_ref() {
        use crate::core::color::Color;
        use crate::core::geometry::Circle;
        use crate::core::math::Vec2;

        let circle = Circle::new(Vec2::new(50.0, 75.0), 25.0, Color::GREEN);
        let instance: CircleInstance = (&circle).into();

        assert_eq!(instance.position, [50.0, 75.0]);
        assert_eq!(instance.radius, 25.0);
    }

    #[test]
    fn test_circle_instance_desc_attributes() {
        let desc = CircleInstance::desc();

        assert_eq!(
            desc.array_stride,
            std::mem::size_of::<CircleInstance>() as u64
        );
        assert_eq!(desc.step_mode, wgpu::VertexStepMode::Instance);
        assert_eq!(desc.attributes.len(), 3); // position, radius, color
    }

    // =========================================================================
    // UNIT TESTS - SpriteInstance
    // =========================================================================

    #[test]
    fn test_sprite_instance_new() {
        let si = SpriteInstance::new(
            [100.0, 200.0],
            [32.0, 64.0],
            [0.0, 0.0, 0.5, 0.5],
            [1.0, 1.0, 1.0, 0.5],
            std::f32::consts::PI,
            42,
        );

        assert_eq!(si.position, [100.0, 200.0]);
        assert_eq!(si.size, [32.0, 64.0]);
        assert_eq!(si.uv_rect, [0.0, 0.0, 0.5, 0.5]);
        assert_eq!(si.tint, [1.0, 1.0, 1.0, 0.5]);
        assert!((si.rotation - std::f32::consts::PI).abs() < 0.0001);
        assert_eq!(si.z_order, 42);
    }

    #[test]
    fn test_sprite_instance_to_gpu_conversion() {
        let si = SpriteInstance::new(
            [1.0, 2.0],
            [3.0, 4.0],
            [0.1, 0.2, 0.3, 0.4],
            [0.5, 0.6, 0.7, 0.8],
            0.9,
            999,
        );

        let gpu = si.to_gpu();

        assert_eq!(gpu.position, si.position);
        assert_eq!(gpu.size, si.size);
        assert_eq!(gpu.uv_rect, si.uv_rect);
        assert_eq!(gpu.tint, si.tint);
        assert_eq!(gpu.rotation, si.rotation);
        // z_order is intentionally not in GPU struct
    }

    #[test]
    fn test_sprite_instance_desc_attributes() {
        let desc = SpriteInstance::desc();

        assert_eq!(
            desc.array_stride,
            std::mem::size_of::<SpriteInstanceGpu>() as u64
        );
        assert_eq!(desc.step_mode, wgpu::VertexStepMode::Instance);
        assert_eq!(desc.attributes.len(), 5); // position, size, uv_rect, tint, rotation
    }

    // =========================================================================
    // REGRESSION TESTS - Memory Layout
    // =========================================================================

    #[test]
    fn test_vertex_size() {
        // 2 floats = 8 bytes
        assert_eq!(std::mem::size_of::<Vertex>(), 8);
    }

    #[test]
    fn test_circle_instance_size() {
        // position: 8 + radius: 4 + color: 16 = 28 bytes
        assert_eq!(std::mem::size_of::<CircleInstance>(), 28);
    }

    #[test]
    fn test_sprite_instance_size() {
        // position: 8 + size: 8 + uv_rect: 16 + tint: 16 + rotation: 4 + z_order: 4 = 56 bytes
        assert_eq!(std::mem::size_of::<SpriteInstance>(), 56);
    }

    #[test]
    fn test_sprite_instance_gpu_size() {
        // position: 8 + size: 8 + uv_rect: 16 + tint: 16 + rotation: 4 = 52 bytes
        assert_eq!(std::mem::size_of::<SpriteInstanceGpu>(), 52);
    }

    #[test]
    fn test_bytemuck_nounit_impl() {
        // Verify bytemuck can safely cast these types
        let vertex = Vertex { pos: [1.0, 2.0] };
        let bytes: &[u8] = bytemuck::bytes_of(&vertex);
        assert_eq!(bytes.len(), 8);

        let circle = CircleInstance::new([0.0, 0.0], 1.0, [1.0; 4]);
        let bytes: &[u8] = bytemuck::bytes_of(&circle);
        assert_eq!(bytes.len(), 28);

        let sprite_gpu = SpriteInstanceGpu {
            position: [0.0, 0.0],
            size: [1.0, 1.0],
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0; 4],
            rotation: 0.0,
        };
        let bytes: &[u8] = bytemuck::bytes_of(&sprite_gpu);
        assert_eq!(bytes.len(), 52);
    }

    #[test]
    fn test_bytemuck_cast_slice() {
        let instances = vec![
            CircleInstance::new([0.0, 0.0], 1.0, [1.0; 4]),
            CircleInstance::new([1.0, 1.0], 2.0, [0.5; 4]),
        ];

        let bytes: &[u8] = bytemuck::cast_slice(&instances);
        assert_eq!(bytes.len(), 56); // 28 * 2

        let sprite_gpus = vec![
            SpriteInstanceGpu {
                position: [0.0, 0.0],
                size: [1.0, 1.0],
                uv_rect: [0.0; 4],
                tint: [1.0; 4],
                rotation: 0.0,
            },
            SpriteInstanceGpu {
                position: [1.0, 1.0],
                size: [2.0, 2.0],
                uv_rect: [0.0; 4],
                tint: [1.0; 4],
                rotation: 0.5,
            },
        ];

        let bytes: &[u8] = bytemuck::cast_slice(&sprite_gpus);
        assert_eq!(bytes.len(), 104); // 52 * 2
    }
}
