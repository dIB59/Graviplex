#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub id: u32,
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub mass: f32,
    pub color: [u8; 4],
    pub radius: f32,
}

impl Body {
    pub fn new(
        id: u32,
        position: [f32; 2],
        velocity: [f32; 2],
        mass: f32,
        color: [u8; 4],
        radius: f32,
    ) -> Self {
        Self {
            id,
            position,
            velocity,
            mass,
            color,
            radius,
        }
    }
}