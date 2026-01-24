//! Example demonstrating the new Schedule-based ECS architecture.
//!
//! This example shows how to use the new system scheduling features:
//! - Schedule with CoreStages (First, PreUpdate, Update, PostUpdate, Last)
//! - Systems as plain functions with World access
//! - Commands for deferred entity operations
//! - StartupSchedule for one-time initialization
//!
//! Run with: cargo run --example schedule_demo

use graviplex::prelude::*;
use graviplex::{CoreStage, Schedule, StartupSchedule};

// =============================================================================
// COMPONENTS
// =============================================================================

/// Marker component for the player entity.
#[derive(Clone)]
struct Player;

/// Marker component for enemy entities.
#[derive(Clone)]
struct Enemy;

/// Score resource stored in the World.
#[derive(Clone, Default)]
struct Score(u32);

/// Game timer resource.
#[derive(Clone)]
struct GameTimer {
    elapsed: f32,
    spawn_cooldown: f32,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            elapsed: 0.0,
            spawn_cooldown: 0.0,
        }
    }
}

// =============================================================================
// SYSTEMS
// =============================================================================

/// Startup system: spawns the player.
fn spawn_player(world: &mut World) {
    println!("Spawning player...");
    world.spawn((
        Transform::from_position(Vec2::ZERO),
        Velocity(Vec2::ZERO),
        Sprite::circle(30.0, Color::BLUE),
        Player,
        Visible,
    ));
}

/// Startup system: initializes resources.
fn init_resources(world: &mut World) {
    println!("Initializing resources...");
    world.insert_resource(Score::default());
    world.insert_resource(GameTimer::default());
}

/// PreUpdate system: processes player input.
fn player_input(world: &mut World) {
    // In a real game, you'd read from InputState resource
    // For this example, we just demonstrate the pattern
    for (_, (vel, _)) in world.query::<(&mut Velocity, &Player)>().iter() {
        // Simulate some input - move right
        // Note: In real code you'd mutate vel here
        let _ = vel; // Acknowledge the mutable borrow
    }
}

/// Update system: moves entities based on velocity.
fn movement(world: &mut World) {
    let dt = 1.0 / 60.0; // In real game, get from Time resource
    for (_, (transform, velocity)) in world.query::<(&mut Transform, &Velocity)>().iter() {
        // Demonstrate the mutable borrow pattern
        let _ = (transform, velocity, dt);
    }
}

/// Update system: checks for collisions and awards points.
fn check_collisions(world: &mut World) {
    // Find player position
    let player_pos = {
        let mut pos = None;
        for (_, (transform, _)) in world.query::<(&Transform, &Player)>().iter() {
            pos = Some(transform.position);
            break;
        }
        pos
    };
    
    let Some(player_pos) = player_pos else { return };
    
    // Check against enemies
    let mut score_delta = 0u32;
    let mut to_despawn = Vec::new();
    
    for (entity, (transform, _)) in world.query::<(&Transform, &Enemy)>().iter() {
        let dist = (transform.position - player_pos).length();
        if dist < 40.0 { // Player radius + Enemy radius
            score_delta += 10;
            to_despawn.push(entity);
        }
    }
    
    // Update score
    if score_delta > 0 {
        let score = world.resource_mut::<Score>();
        score.0 += score_delta;
        println!("Score: {}", score.0);
    }
    
    // Mark enemies for despawn
    for entity in to_despawn {
        let _ = world.insert(entity, Despawn);
    }
}

/// PostUpdate system: spawns new enemies periodically.
fn spawn_enemies(world: &mut World) {
    let dt = 1.0 / 60.0;
    
    // Update timer
    {
        let timer = world.resource_mut::<GameTimer>();
        timer.elapsed += dt;
        timer.spawn_cooldown -= dt;
    }
    
    // Check if we should spawn
    let should_spawn = world.resource::<GameTimer>().spawn_cooldown <= 0.0;
    
    if should_spawn {
        // Reset cooldown
        world.resource_mut::<GameTimer>().spawn_cooldown = 2.0;
        
        // Spawn enemy at random-ish position (using elapsed time as seed)
        let elapsed = world.resource::<GameTimer>().elapsed;
        let x = (elapsed * 100.0).sin() * 200.0;
        let y = (elapsed * 73.0).cos() * 150.0;
        
        world.spawn((
            Transform::from_position(Vec2::new(x, y)),
            Sprite::circle(20.0, Color::RED),
            Enemy,
            Visible,
        ));
        
        println!("Spawned enemy at ({:.1}, {:.1})", x, y);
    }
}

/// Last system: cleans up despawned entities.
fn cleanup(world: &mut World) {
    despawn_system(world);
}

// =============================================================================
// GAME LOOP
// =============================================================================

struct ScheduleDemo {
    schedule: Schedule,
    startup: StartupSchedule,
    frame_count: u32,
}

impl ScheduleDemo {
    fn new() -> Self {
        // Create the main schedule with systems in different stages
        let mut schedule = Schedule::new();
        
        // PreUpdate: Input processing
        schedule.add_system(CoreStage::PreUpdate, player_input);
        
        // Update: Game logic
        schedule.add_system(CoreStage::Update, movement);
        schedule.add_system(CoreStage::Update, check_collisions);
        
        // PostUpdate: Spawning, physics response
        schedule.add_system(CoreStage::PostUpdate, spawn_enemies);
        
        // Last: Cleanup
        schedule.add_system(CoreStage::Last, cleanup);
        
        // Create startup schedule
        let mut startup = StartupSchedule::new();
        startup.add_system(init_resources);
        startup.add_system(spawn_player);
        
        Self {
            schedule,
            startup,
            frame_count: 0,
        }
    }
}

impl GameLoop for ScheduleDemo {
    fn init(&mut self, world: &mut World, _gfx: &Graphics) {
        // Run startup systems (only runs once)
        self.startup.run(world);
        println!("\nStarting game loop...\n");
    }
    
    fn update(&mut self, world: &mut World, _res: &Resources) {
        // Run the schedule (all stages in order)
        self.schedule.run(world);
        
        self.frame_count += 1;
        
        // Print stats every 60 frames
        if self.frame_count % 60 == 0 {
            let entity_count = world.len();
            let score = world.resource::<Score>().0;
            println!("Frame {}: {} entities, Score: {}", self.frame_count, entity_count, score);
        }
    }
    
    fn render(&mut self, world: &World, draw: &mut DrawContext) {
        // Background is handled by the window clear color
        draw.render_world(world);
    }
}

// =============================================================================
// MAIN
// =============================================================================

fn main() {
    println!("Schedule Demo - New ECS Architecture");
    println!("=====================================");
    println!();
    println!("This example demonstrates:");
    println!("- Schedule with CoreStages");
    println!("- StartupSchedule for initialization");
    println!("- Systems as plain functions");
    println!();
    
    App::build(ScheduleDemo::new())
        .title("Schedule Demo")
        .size(800, 600)
        .run()
        .expect("Failed to run app");
}
