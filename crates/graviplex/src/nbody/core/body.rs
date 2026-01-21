#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub id: u32,
    pub position: [f64; 2],
    pub velocity: [f64; 2],
    pub mass: f64,
    pub color: [u8; 4],
    pub radius: f64,
}

impl Body {
    pub fn new(
        id: u32,
        position: [f64; 2],
        velocity: [f64; 2],
        mass: f64,
        color: [u8; 4],
        radius: f64,
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
