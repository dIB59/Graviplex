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
}

impl SimulationBridge {
    pub fn new(count: i32) -> Self {
        let (tx, rx) = mpsc::channel();
        let buffer = Arc::new(TripleBuffer::new());

        let worker_buffer = buffer.clone();
        std::thread::spawn(move || {
            simulation_worker(rx, worker_buffer, count);
        });

        Self { tx, buffer }
    }

    pub fn get_instances(&self) -> Arc<RwLock<Vec<Instance>>> {
        self.buffer.fetch()
    }

    pub fn sender(&self) -> Sender<SimulationCommand> {
        self.tx.clone()
    }
}

fn simulation_worker(
    rx: Receiver<SimulationCommand>,
    buffer: Arc<TripleBuffer<Vec<Instance>>>,
    initial_count: i32,
) {
    let mut sim = Simulation::default();
    sim.generate_bodies(initial_count);

    let mut last_tick = Instant::now();
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
            }
        }

        // 2. Update simulation
        let now = Instant::now();
        let dt = now.duration_since(last_tick).as_secs_f64();
        last_tick = now;

        if !is_paused {
            sim.update(dt);
        }

        // 3. Convert to instances and submit
        let instances: Vec<Instance> = sim.bodies().iter().map(Instance::from).collect();
        buffer.submit(instances);

        // Optional: yield to prevent 100% CPU on spin-lock if simulation is ultra fast
        // std::thread::yield_now();
    }
}
