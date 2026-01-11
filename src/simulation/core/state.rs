use super::body::Body;

/// SimulationState handles the data layout for the physics engine.
/// It uses a semi-SoA (Structure of Arrays) approach to improve cache locality.
pub struct SimulationState {
    pub ids: Vec<u32>,
    pub positions: Vec<[f64; 2]>,
    pub velocities: Vec<[f64; 2]>,
    pub masses: Vec<f64>,
    pub colors: Vec<[u8; 4]>,
    pub radii: Vec<f64>,
}

impl SimulationState {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            masses: Vec::new(),
            colors: Vec::new(),
            radii: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            masses: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            radii: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, body: Body) {
        self.ids.push(body.id);
        self.positions.push(body.position);
        self.velocities.push(body.velocity);
        self.masses.push(body.mass);
        self.colors.push(body.color);
        self.radii.push(body.radius);
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.positions.clear();
        self.velocities.clear();
        self.masses.clear();
        self.colors.clear();
        self.radii.clear();
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Converts back to AoS for external consumption or rendering if necessary.
    pub fn to_bodies(&self) -> Vec<Body> {
        (0..self.len())
            .map(|i| Body {
                id: self.ids[i],
                position: self.positions[i],
                velocity: self.velocities[i],
                mass: self.masses[i],
                color: self.colors[i],
                radius: self.radii[i],
            })
            .collect()
    }
}
