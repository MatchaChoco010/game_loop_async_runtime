use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use runtime_v5::{Runtime, RuntimeIsDone};

mod enemy_system;
mod input_system;
mod key_events;
mod late_update_system;
mod player_system;
mod render_system;
mod world;

use enemy_system::enemy_system;
use input_system::input_system;
use late_update_system::late_update_system;
use player_system::player_system;
use render_system::render_system;
use world::World;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Phase {
    Input,
    Update,
    LateUpdate,
    Render,
}

fn main() {
    let world = Arc::new(Mutex::new(World::new()));
    let stdout = Arc::new(Mutex::new(stdout()));

    let mut runtime = Runtime::new();

    runtime.activate_phase(Phase::Input, 0);
    runtime.activate_phase(Phase::Update, 10);
    runtime.activate_phase(Phase::LateUpdate, 30);
    runtime.activate_phase(Phase::Render, 40);

    runtime.spawn(Phase::Input, input_system(world.clone()));
    runtime.spawn(Phase::Update, player_system(world.clone()));
    runtime.spawn(Phase::Update, enemy_system(world.clone()));
    runtime.spawn(Phase::LateUpdate, late_update_system(world.clone()));
    runtime.spawn(Phase::Render, render_system(world.clone(), stdout));

    enable_raw_mode().unwrap();

    'update_loop: loop {
        let frame_start = Instant::now();
        let frame_duration = Duration::from_millis(83);

        match runtime.update() {
            RuntimeIsDone::Done => break 'update_loop,
            RuntimeIsDone::NotDone => (),
        }

        let now = Instant::now();
        let duration = now.duration_since(frame_start);
        if duration < frame_duration {
            sleep(frame_duration - duration);
        }
    }

    disable_raw_mode().unwrap();
}
