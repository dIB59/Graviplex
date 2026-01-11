use crate::renderer::Instance;
use crate::simulation::{BarnesHutGravityStrategy, GravityStrategyEnum, Simulation};
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

/// Quad cell data for visualization (center and size)
#[derive(Clone, Default)]
pub struct QuadCell {
    pub center: [f32; 2],
    pub size: f32,
}

pub struct SimulationBridge {
    tx: Sender<SimulationCommand>,
    buffer: Arc<TripleBuffer<Vec<Instance>>>,
    quad_cells_buffer: Arc<TripleBuffer<Vec<QuadCell>>>,
    tps: Arc<RwLock<f32>>,
    current_body_count: Arc<RwLock<usize>>,
}

impl SimulationBridge {
    pub fn new(count: i32) -> Self {
        let (tx, rx) = mpsc::channel();
        let buffer = Arc::new(TripleBuffer::new());
        let quad_cells_buffer = Arc::new(TripleBuffer::new());
        let tps = Arc::new(RwLock::new(0.0));
        let current_body_count = Arc::new(RwLock::new(count as usize));

        let worker_buffer = buffer.clone();
        let worker_quad_cells = quad_cells_buffer.clone();
        let worker_tps = tps.clone();
        let worker_count = current_body_count.clone();
        std::thread::spawn(move || {
            simulation_worker(
                rx,
                worker_buffer,
                worker_quad_cells,
                worker_tps,
                worker_count,
                count,
            );
        });

        Self {
            tx,
            buffer,
            quad_cells_buffer,
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

    pub fn get_quad_cells(&self) -> Arc<RwLock<Vec<QuadCell>>> {
        self.quad_cells_buffer.fetch()
    }
}

fn simulation_worker(
    rx: Receiver<SimulationCommand>,
    buffer: Arc<TripleBuffer<Vec<Instance>>>,
    quad_cells_buffer: Arc<TripleBuffer<Vec<QuadCell>>>,
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
                    sim.set_gravity_strategy(GravityStrategyEnum::BarnesHut(
                        BarnesHutGravityStrategy::new(theta, 0.01),
                    ));
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
        let dt = now.duration_since(last_tick).as_secs_f32();
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
        let instances: Vec<Instance> = sim.state.to_instances();
        let count = instances.len();
        buffer.submit(instances);

        // Update body count periodically or on change
        {
            let mut count_lock = body_count_shared.write().unwrap();
            if *count_lock != count {
                *count_lock = count;
            }
        }

        // 4. Convert quad cells and submit
        let quad_cells: Vec<QuadCell> = sim
            .get_cells()
            .iter()
            .map(|q| QuadCell {
                center: [q.center[0] as f32, q.center[1] as f32],
                size: q.size as f32,
            })
            .collect();
        quad_cells_buffer.submit(quad_cells);

        // Optional: yield to prevent 100% CPU on spin-lock if simulation is ultra fast
        // std::thread::yield_now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // Tolerance for floating-point position comparisons in tests
    const POSITION_EPSILON: f32 = 0.0001;

    // Helper function to get expected body count
    // generate_bodies creates count + 1 bodies (1 central body + count orbiting bodies)
    const fn expected_body_count(requested_count: i32) -> usize {
        (requested_count + 1) as usize
    }

    #[test]
    fn test_triple_buffer_basic_submit_and_fetch() {
        let buffer = TripleBuffer::<Vec<i32>>::new();

        // Submit some data
        buffer.submit(vec![1, 2, 3]);

        // Fetch should return the submitted data
        let fetched = buffer.fetch();
        let data = fetched.read().unwrap();
        assert_eq!(*data, vec![1, 2, 3]);
    }

    #[test]
    fn test_triple_buffer_multiple_submits() {
        let buffer = TripleBuffer::<Vec<i32>>::new();

        // Submit multiple times
        buffer.submit(vec![1, 2, 3]);
        buffer.submit(vec![4, 5, 6]);
        buffer.submit(vec![7, 8, 9]);

        // Fetch should return the latest submitted data
        let fetched = buffer.fetch();
        let data = fetched.read().unwrap();
        assert_eq!(*data, vec![7, 8, 9]);
    }

    #[test]
    fn test_triple_buffer_dirty_flag() {
        let buffer = TripleBuffer::<Vec<i32>>::new();

        // Initially dirty should be false (or true after first submit)
        buffer.submit(vec![1, 2, 3]);

        // After submit, dirty should be true
        assert!(*buffer.dirty.read().unwrap());

        // After fetch, dirty should be false
        buffer.fetch();
        assert!(!*buffer.dirty.read().unwrap());

        // Fetch again without submit should keep dirty false
        buffer.fetch();
        assert!(!*buffer.dirty.read().unwrap());
    }

    #[test]
    fn test_triple_buffer_no_swap_when_not_dirty() {
        let buffer = TripleBuffer::<Vec<i32>>::new();

        // Submit and fetch once
        buffer.submit(vec![1, 2, 3]);
        let first_fetch = buffer.fetch();
        let front_ptr_1 = Arc::as_ptr(&first_fetch);

        // Fetch again without submitting - should return same front buffer
        let second_fetch = buffer.fetch();
        let front_ptr_2 = Arc::as_ptr(&second_fetch);

        assert_eq!(front_ptr_1, front_ptr_2);
    }

    #[test]
    fn test_triple_buffer_concurrent_access() {
        let buffer = Arc::new(TripleBuffer::<Vec<i32>>::new());

        // Spawn writer thread
        let writer_buffer = buffer.clone();
        let writer = thread::spawn(move || {
            for i in 0..10 {
                writer_buffer.submit(vec![i]);
                thread::sleep(Duration::from_millis(10));
            }
        });

        // Spawn reader thread
        let reader_buffer = buffer.clone();
        let reader = thread::spawn(move || {
            let mut last_seen = -1;
            for _ in 0..10 {
                let fetched = reader_buffer.fetch();
                let data = fetched.read().unwrap();
                if !data.is_empty() {
                    let value = data[0];
                    // Values should be monotonically increasing or same
                    assert!(value >= last_seen);
                    last_seen = value;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        writer.join().unwrap();
        reader.join().unwrap();
    }

    #[test]
    fn test_triple_buffer_buffer_rotation() {
        let buffer = TripleBuffer::<Vec<i32>>::new();

        // Track initial indices
        let initial_front = *buffer.front_idx.lock().unwrap();
        let initial_mid = *buffer.mid_idx.lock().unwrap();
        let initial_back = *buffer.back_idx.lock().unwrap();

        // All indices should be different
        assert_ne!(initial_front, initial_mid);
        assert_ne!(initial_front, initial_back);
        assert_ne!(initial_mid, initial_back);

        // Submit should swap back and mid
        buffer.submit(vec![1]);
        let after_submit_mid = *buffer.mid_idx.lock().unwrap();
        let after_submit_back = *buffer.back_idx.lock().unwrap();
        assert_eq!(after_submit_mid, initial_back);
        assert_eq!(after_submit_back, initial_mid);

        // Fetch should swap front and mid when dirty
        buffer.fetch();
        let after_fetch_front = *buffer.front_idx.lock().unwrap();
        let after_fetch_mid = *buffer.mid_idx.lock().unwrap();
        assert_eq!(after_fetch_front, after_submit_mid);
        assert_eq!(after_fetch_mid, initial_front);
    }

    #[test]
    fn test_simulation_bridge_creation() {
        let bridge = SimulationBridge::new(10);

        // Give the simulation thread time to start
        thread::sleep(Duration::from_millis(100));

        // Should be able to get instances
        let instances = bridge.get_instances();
        let data = instances.read().unwrap();

        assert_eq!(data.len(), expected_body_count(10));
    }

    #[test]
    fn test_simulation_bridge_get_body_count() {
        let bridge = SimulationBridge::new(5);

        // Give the simulation thread time to start
        thread::sleep(Duration::from_millis(100));

        assert_eq!(bridge.get_body_count(), expected_body_count(5));
    }

    #[test]
    fn test_simulation_bridge_reset_command() {
        let bridge = SimulationBridge::new(10);

        // Give the simulation thread time to start
        thread::sleep(Duration::from_millis(100));

        // Send reset command with different count
        bridge.sender().send(SimulationCommand::Reset(20)).unwrap();

        // Wait for command to be processed
        thread::sleep(Duration::from_millis(200));

        assert_eq!(bridge.get_body_count(), expected_body_count(20));

        let instances = bridge.get_instances();
        let data = instances.read().unwrap();
        assert_eq!(data.len(), expected_body_count(20));
    }

    #[test]
    fn test_simulation_bridge_update_gravity_command() {
        let bridge = SimulationBridge::new(5);

        // Give the simulation thread time to start
        thread::sleep(Duration::from_millis(100));

        // Send update gravity command - should not panic
        bridge
            .sender()
            .send(SimulationCommand::UpdateGravity(100.0))
            .unwrap();

        // Wait for command to be processed
        thread::sleep(Duration::from_millis(100));

        // Simulation should still be running
        let instances = bridge.get_instances();
        let data = instances.read().unwrap();
        assert_eq!(data.len(), expected_body_count(5));
    }

    #[test]
    fn test_simulation_bridge_set_theta_command() {
        let bridge = SimulationBridge::new(5);

        // Give the simulation thread time to start
        thread::sleep(Duration::from_millis(100));

        // Send set theta command - should not panic
        bridge
            .sender()
            .send(SimulationCommand::SetTheta(0.8))
            .unwrap();

        // Wait for command to be processed
        thread::sleep(Duration::from_millis(100));

        // Simulation should still be running
        let instances = bridge.get_instances();
        let data = instances.read().unwrap();
        assert_eq!(data.len(), expected_body_count(5));
    }

    #[test]
    fn test_simulation_bridge_pause_command() {
        let bridge = SimulationBridge::new(5);

        // Give the simulation thread time to start and run a bit
        thread::sleep(Duration::from_millis(100));

        // Pause the simulation
        bridge
            .sender()
            .send(SimulationCommand::Pause(true))
            .unwrap();
        thread::sleep(Duration::from_millis(100));

        // Get instances after pause
        let instances_after_pause = bridge.get_instances();
        let positions_after_pause: Vec<_> = instances_after_pause
            .read()
            .unwrap()
            .iter()
            .map(|i| i.position)
            .collect();

        // Wait a bit more
        thread::sleep(Duration::from_millis(200));

        // Positions should remain the same when paused
        let instances_final = bridge.get_instances();
        let positions_final: Vec<_> = instances_final
            .read()
            .unwrap()
            .iter()
            .map(|i| i.position)
            .collect();

        // While paused, positions should not change significantly
        // (allowing for floating point precision)
        for (pos1, pos2) in positions_after_pause.iter().zip(positions_final.iter()) {
            assert!((pos1[0] - pos2[0]).abs() < POSITION_EPSILON);
            assert!((pos1[1] - pos2[1]).abs() < POSITION_EPSILON);
        }

        // Unpause
        bridge
            .sender()
            .send(SimulationCommand::Pause(false))
            .unwrap();
        thread::sleep(Duration::from_millis(100));
    }

    #[test]
    fn test_simulation_bridge_step_command() {
        let bridge = SimulationBridge::new(5);

        // Give the simulation thread time to start
        thread::sleep(Duration::from_millis(100));

        // Pause the simulation first
        bridge
            .sender()
            .send(SimulationCommand::Pause(true))
            .unwrap();
        thread::sleep(Duration::from_millis(100));

        // Send step command - should advance by one fixed step
        bridge.sender().send(SimulationCommand::Step).unwrap();

        // Wait for command to be processed
        thread::sleep(Duration::from_millis(100));

        // Simulation should still have same number of bodies
        let instances = bridge.get_instances();
        let data = instances.read().unwrap();
        assert_eq!(data.len(), expected_body_count(5));
    }

    #[test]
    fn test_simulation_bridge_tps_tracking() {
        let bridge = SimulationBridge::new(5);

        // Wait for at least one TPS calculation (happens every second)
        thread::sleep(Duration::from_millis(1100));

        // TPS should be greater than 0 (simulation is running)
        let tps = bridge.get_tps();
        assert!(tps > 0.0);
    }

    #[test]
    fn test_simulation_bridge_multiple_commands() {
        let bridge = SimulationBridge::new(10);

        // Give the simulation thread time to start
        thread::sleep(Duration::from_millis(100));

        // Send multiple commands in sequence
        bridge
            .sender()
            .send(SimulationCommand::UpdateGravity(50.0))
            .unwrap();
        bridge
            .sender()
            .send(SimulationCommand::SetTheta(0.9))
            .unwrap();
        bridge.sender().send(SimulationCommand::Reset(15)).unwrap();

        // Wait for all commands to be processed
        thread::sleep(Duration::from_millis(300));

        assert_eq!(bridge.get_body_count(), expected_body_count(15));

        let instances = bridge.get_instances();
        let data = instances.read().unwrap();
        assert_eq!(data.len(), expected_body_count(15));
    }

    #[test]
    fn test_simulation_bridge_sender_cloning() {
        let bridge = SimulationBridge::new(5);

        // Give the simulation thread time to start
        thread::sleep(Duration::from_millis(100));

        // Clone the sender
        let sender1 = bridge.sender();
        let sender2 = bridge.sender();

        // Both senders should work
        sender1
            .send(SimulationCommand::UpdateGravity(10.0))
            .unwrap();
        sender2
            .send(SimulationCommand::UpdateGravity(20.0))
            .unwrap();

        // Wait for commands to be processed
        thread::sleep(Duration::from_millis(100));

        // Simulation should still be running
        let instances = bridge.get_instances();
        let data = instances.read().unwrap();
        assert_eq!(data.len(), expected_body_count(5));
    }
}
