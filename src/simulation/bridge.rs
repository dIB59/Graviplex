use crate::renderer::Instance;
use crate::simulation::{BarnesHutGravityStrategy, Simulation};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

pub enum SimulationCommand {
    UpdateGravity(f64),
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
/// rendering_thread: reads from 'front'
/// simulation_thread: writes to 'back'
/// 'mid' is used for swapping
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

    /// Called by the simulation thread to submit a new state.
    pub fn submit(&self, value: T) {
        let back = *self.back_idx.lock().unwrap();
        *self.buffers[back].write().unwrap() = value;

        // Swap back with mid
        let mut mid_lock = self.mid_idx.lock().unwrap();
        let mut back_lock = self.back_idx.lock().unwrap();
        let temp = *mid_lock;
        *mid_lock = *back_lock;
        *back_lock = temp;

        *self.dirty.write().unwrap() = true;
    }

    /// Called by the rendering thread to fetch the latest state.
    pub fn fetch(&self) -> Arc<RwLock<T>> {
        if *self.dirty.read().unwrap() {
            let mut mid_lock = self.mid_idx.lock().unwrap();
            let mut front_lock = self.front_idx.lock().unwrap();

            // Swap front with mid
            let temp = *mid_lock;
            *mid_lock = *front_lock;
            *front_lock = temp;

            *self.dirty.write().unwrap() = false;
        }

        let front = *self.front_idx.lock().unwrap();
        self.buffers[front].clone()
    }
}

pub struct SimulationBridge {
    tx: Sender<SimulationCommand>,
    buffer: Arc<TripleBuffer<Vec<Instance>>>,
    tps: Arc<RwLock<f32>>,
    current_body_count: Arc<RwLock<usize>>,
}

impl SimulationBridge {
    pub fn new(count: i32) -> Self {
        let (tx, rx) = mpsc::channel();
        let buffer = Arc::new(TripleBuffer::new());
        let tps = Arc::new(RwLock::new(0.0));
        let current_body_count = Arc::new(RwLock::new(count as usize));

        let worker_buffer = buffer.clone();
        let worker_tps = tps.clone();
        let worker_count = current_body_count.clone();
        std::thread::spawn(move || {
            simulation_worker(rx, worker_buffer, worker_tps, worker_count, count);
        });

        Self {
            tx,
            buffer,
            tps,
            current_body_count,
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
}

fn simulation_worker(
    rx: Receiver<SimulationCommand>,
    buffer: Arc<TripleBuffer<Vec<Instance>>>,
    tps_shared: Arc<RwLock<f32>>,
    body_count_shared: Arc<RwLock<usize>>,
    initial_count: i32,
) {
    let mut sim = Simulation::default();
    sim.generate_bodies(initial_count);

    let mut last_tick = Instant::now();
    let mut last_tps_check = Instant::now();
    let mut update_count = 0;
    let mut is_paused = false;

    loop {
        // 1. Process commands
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                SimulationCommand::UpdateGravity(g) => sim.gravity_constant = g,
                SimulationCommand::Reset(count) => {
                    sim.clear();
                    sim.generate_bodies(count);
                }
                SimulationCommand::SetTheta(theta) => {
                    sim.set_gravity_strategy(Box::new(BarnesHutGravityStrategy::new(theta, 0.01)));
                }
                SimulationCommand::Pause(p) => is_paused = p,
                SimulationCommand::Step => {
                    sim.update(0.016); // fixed step for manual advance
                }
                SimulationCommand::Interaction {
                    pos,
                    radius,
                    strength,
                } => {
                    sim.apply_interaction(pos, radius, strength);
                }
            }
        }

        // 2. Update simulation
        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f64();
        last_tick = now;

        if !is_paused {
            sim.update(dt);
            update_count += 1;
        }

        // Calculate TPS every second
        let time_since_tps = now.duration_since(last_tps_check).as_secs_f32();
        if time_since_tps >= 1.0 {
            let mut tps_lock = tps_shared.write().unwrap();
            *tps_lock = update_count as f32 / time_since_tps;
            update_count = 0;
            last_tps_check = now;
        }

        // 3. Convert to instances and submit
        let instances: Vec<Instance> = sim.bodies().iter().map(Instance::from).collect();
        let count = instances.len();
        buffer.submit(instances);

        // Update body count periodically or on change
        {
            let mut count_lock = body_count_shared.write().unwrap();
            if *count_lock != count {
                *count_lock = count;
            }
        }

        // Optional: yield to prevent 100% CPU on spin-lock if simulation is ultra fast
        // std::thread::yield_now();
    }
}
