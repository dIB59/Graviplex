use crate::renderer::Instance;
use crate::simulation::Simulation;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

pub enum SimulationCommand {
    UpdateGravity(f32),
    Reset(i32),
    SetTheta(f64),
    Pause(bool),
    Step,
    Interaction {
        pos: [f64; 2],
        radius: f64,
        strength: f64,
    },
}

/// A Triple Buffer for zero-contention state sharing.
pub struct TripleBuffer<T> {
    buffers: [Arc<RwLock<T>>; 3],
    front_idx: Mutex<usize>,
    back_idx: Mutex<usize>,
    mid_idx: Mutex<usize>,
    dirty: Arc<RwLock<bool>>,
}

impl<T: Clone + Default> TripleBuffer<T> {
    pub fn new() -> Self {
        Self {
            buffers: [
                Arc::new(RwLock::new(T::default())),
                Arc::new(RwLock::new(T::default())),
                Arc::new(RwLock::new(T::default())),
            ],
            front_idx: Mutex::new(0),
            mid_idx: Mutex::new(1),
            back_idx: Mutex::new(2),
            dirty: Arc::new(RwLock::new(false)),
        }
    }

    pub fn submit(&self, value: T) {
        let back = *self.back_idx.lock().unwrap();
        *self.buffers[back].write().unwrap() = value;

        let mut mid_lock = self.mid_idx.lock().unwrap();
        let mut back_lock = self.back_idx.lock().unwrap();
        let temp = *mid_lock;
        *mid_lock = *back_lock;
        *back_lock = temp;

        *self.dirty.write().unwrap() = true;
    }

    pub fn fetch(&self) -> Arc<RwLock<T>> {
        if *self.dirty.read().unwrap() {
            let mut mid_lock = self.mid_idx.lock().unwrap();
            let mut front_lock = self.front_idx.lock().unwrap();
            let temp = *mid_lock;
            *mid_lock = *front_lock;
            *front_lock = temp;
            *self.dirty.write().unwrap() = false;
        }
        let front = *self.front_idx.lock().unwrap();
        self.buffers[front].clone()
    }
}

#[derive(Clone)]
pub struct QuadCell {
    pub center: [f32; 2],
    pub size: f32,
}

impl Default for QuadCell {
    fn default() -> Self {
        Self {
            center: [0.0, 0.0],
            size: 0.0,
        }
    }
}

pub struct SimulationBridge {
    tx: Sender<SimulationCommand>,
    buffer: Arc<TripleBuffer<Vec<Instance>>>,
    quad_cells_buffer: Arc<TripleBuffer<Vec<QuadCell>>>,
    tps: Arc<RwLock<f32>>,
    current_body_count: Arc<RwLock<usize>>,
    initial_bodies: Arc<RwLock<Vec<crate::simulation::core::Body>>>,
}

impl SimulationBridge {
    pub fn new(count: i32) -> Self {
        let (tx, rx) = mpsc::channel();
        let buffer = Arc::new(TripleBuffer::new());
        let quad_cells_buffer = Arc::new(TripleBuffer::new());
        let tps = Arc::new(RwLock::new(0.0));
        let current_body_count = Arc::new(RwLock::new(count as usize));

        let mut sim = Simulation::default();
        sim.generate_bodies(count);
        let initial_bodies = Arc::new(RwLock::new(sim.state.to_bodies()));

        let worker_quad_cells = quad_cells_buffer.clone();
        let worker_tps = tps.clone();
        let worker_count = current_body_count.clone();
        let worker_initial = initial_bodies.clone();

        std::thread::spawn(move || {
            simulation_worker(
                rx,
                worker_quad_cells,
                worker_tps,
                worker_count,
                worker_initial,
            );
        });

        Self {
            tx,
            buffer,
            quad_cells_buffer,
            tps,
            current_body_count,
            initial_bodies,
        }
    }

    pub fn get_instances(&self) -> Arc<RwLock<Vec<Instance>>> {
        self.buffer.fetch()
    }

    pub fn sender(&self) -> Sender<SimulationCommand> {
        self.tx.clone()
    }

    pub fn get_tps(&self) -> f32 {
        *self.tps.read().unwrap()
    }

    pub fn get_body_count(&self) -> usize {
        *self.current_body_count.read().unwrap()
    }

    pub fn get_quad_cells(&self) -> Arc<RwLock<Vec<QuadCell>>> {
        self.quad_cells_buffer.fetch()
    }

    pub fn get_initial_bodies(&self) -> Vec<crate::simulation::core::Body> {
        self.initial_bodies.read().unwrap().clone()
    }
}

fn simulation_worker(
    rx: Receiver<SimulationCommand>,
    quad_cells_buffer: Arc<TripleBuffer<Vec<QuadCell>>>,
    tps_shared: Arc<RwLock<f32>>,
    body_count_shared: Arc<RwLock<usize>>,
    _initial_bodies: Arc<RwLock<Vec<crate::simulation::core::Body>>>,
) {
    let mut _last_tick = Instant::now();
    let mut last_tps_check = Instant::now();
    let mut update_count = 0;
    let mut is_paused = false;

    loop {
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                SimulationCommand::Pause(p) => is_paused = p,
                SimulationCommand::Reset(count) => {
                    let mut count_lock = body_count_shared.write().unwrap();
                    *count_lock = count as usize;
                }
                _ => {} // Other commands handled by GPU engine or ignored
            }
        }

        let now = Instant::now();
        _last_tick = now;

        if !is_paused {
            update_count += 60; // Approximate for UI
        }

        let time_since_tps = now.duration_since(last_tps_check).as_secs_f32();
        if time_since_tps >= 1.0 {
            let mut tps_lock = tps_shared.write().unwrap();
            *tps_lock = update_count as f32 / time_since_tps;
            update_count = 0;
            last_tps_check = now;
        }

        quad_cells_buffer.submit(vec![]); // Clear CPU quadtree visualization
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
