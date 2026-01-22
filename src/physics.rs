//! Physics simulation types for high-performance particle systems.
//!
//! This module provides GPU-compatible data structures for physics simulations
//! that integrate with the rendering pipeline.

use crate::core::geometry::Circle;
use bytemuck::NoUninit;

/// A physics-enabled instance for GPU particle simulations.
///
/// This struct is designed to match the memory layout of GPU compute shaders
/// for physics simulations while still being renderable by the circle pipeline.
///
/// # Memory Layout
///
/// The struct has a 32-byte stride for GPU alignment:
/// - `position`: 8 bytes (2x f32)
/// - `radius`: 4 bytes (f32)
/// - `color`: 4 bytes (4x u8 normalized)
/// - `velocity`: 8 bytes (2x f32)
/// - `mass`: 4 bytes (f32)
/// - `id`: 4 bytes (u32)
#[repr(C)]
#[derive(Copy, Clone, NoUninit, Debug)]
pub struct PhysicsInstance {
    pub position: [f32; 2],
    pub radius: f32,
    pub color: [u8; 4],
    pub velocity: [f32; 2],
    pub mass: f32,
    pub id: u32,
}

impl PhysicsInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array!(1 => Float32x2, 2 => Float32, 3 => Unorm8x4);

    /// Returns the vertex buffer layout for instanced rendering.
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: 32, // Fixed stride for GPU alignment
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }

    /// Create a new physics instance with default physics properties.
    pub fn new(position: [f32; 2], radius: f32, color: [u8; 4]) -> Self {
        Self {
            position,
            radius,
            color,
            velocity: [0.0, 0.0],
            mass: 1.0,
            id: 0,
        }
    }

    /// Set the velocity of this instance.
    pub fn with_velocity(mut self, velocity: [f32; 2]) -> Self {
        self.velocity = velocity;
        self
    }

    /// Set the mass of this instance.
    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Set the ID of this instance (useful for tracking in simulations).
    pub fn with_id(mut self, id: u32) -> Self {
        self.id = id;
        self
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
