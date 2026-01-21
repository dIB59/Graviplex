use super::body::Body;

/// SimulationState handles the data layout for the physics engine.
/// It uses a "True SoA" (Structure of Arrays) approach where every primitive component
/// stays in its own contiguous vector. This is optimal for CPU SIMD (AVX/SSE)
/// and simplifies GPU buffer uploads.
pub struct SimulationState {
    pub ids: Vec<u32>,
    // Position (f64 for high precision ground truth)
    pub px: Vec<f64>,
    pub py: Vec<f64>,
    // Velocity
    pub vx: Vec<f64>,
    pub vy: Vec<f64>,
    // Acceleration
    pub ax: Vec<f64>,
    pub ay: Vec<f64>,
    pub masses: Vec<f64>,
    pub colors: Vec<[u8; 4]>,
    pub radii: Vec<f64>,
}

impl SimulationState {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            px: Vec::new(),
            py: Vec::new(),
            vx: Vec::new(),
            vy: Vec::new(),
            ax: Vec::new(),
            ay: Vec::new(),
            masses: Vec::new(),
            colors: Vec::new(),
            radii: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            px: Vec::with_capacity(capacity),
            py: Vec::with_capacity(capacity),
            vx: Vec::with_capacity(capacity),
            vy: Vec::with_capacity(capacity),
            ax: Vec::with_capacity(capacity),
            ay: Vec::with_capacity(capacity),
            masses: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            radii: Vec::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, body: Body) {
        self.ids.push(body.id);
        self.px.push(body.position[0]);
        self.py.push(body.position[1]);
        self.vx.push(body.velocity[0]);
        self.vy.push(body.velocity[1]);
        self.ax.push(0.0);
        self.ay.push(0.0);
        self.masses.push(body.mass);
        self.colors.push(body.color);
        self.radii.push(body.radius);
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.px.clear();
        self.py.clear();
        self.vx.clear();
        self.vy.clear();
        self.ax.clear();
        self.ay.clear();
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

    pub fn to_bodies(&self) -> Vec<Body> {
        (0..self.len())
            .map(|i| Body {
                id: self.ids[i],
                position: [self.px[i], self.py[i]],
                velocity: [self.vx[i], self.vy[i]],
                mass: self.masses[i],
                color: self.colors[i],
                radius: self.radii[i],
            })
            .collect()
    }

    /// Optimized conversion direct to render instances.
    /// Converts f64 physics ground-truth to f32 for the GPU.
    pub fn to_instances(&self) -> Vec<graviplex::renderer::Instance> {
        use rayon::prelude::*;
        (0..self.len())
            .into_par_iter()
            .map(|i| graviplex::renderer::Instance {
                position: [self.px[i] as f32, self.py[i] as f32],
                radius: self.radii[i] as f32,
                color: self.colors[i],
                velocity: [self.vx[i] as f32, self.vy[i] as f32],
                mass: self.masses[i] as f32,
                id: self.ids[i],
            })
            .collect()
    }
}
