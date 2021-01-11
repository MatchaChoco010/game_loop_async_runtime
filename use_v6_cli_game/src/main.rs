use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use runtime_v6::{Runtime, RuntimeIsDone};

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
use world::GameWorld;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Phase {
    Input,
    Update,
    LateUpdate,
    Render,
}

fn main() {
    let world = GameWorld::new();

    let mut runtime = Runtime::new(world);

    runtime.activate_phase(Phase::Input, 0);
    runtime.activate_phase(Phase::Update, 10);
    runtime.activate_phase(Phase::LateUpdate, 20);
    runtime.activate_phase(Phase::Render, 30);

    runtime.add_async_system(Phase::Input, input_system);
    runtime.add_async_system(Phase::Update, player_system);
    runtime.add_async_system(Phase::Update, enemy_system);
    runtime.add_async_system(Phase::LateUpdate, late_update_system);
    runtime.add_async_system(Phase::Render, render_system);

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
