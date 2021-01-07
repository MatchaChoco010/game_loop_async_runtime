use std::sync::{Arc, Mutex, RwLock};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{io::stdout, sync::mpsc::channel};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use runtime_v5::{Runtime, RuntimeIsDone};

mod enemy_system;
mod input_system;
mod key_events;
mod late_update_system;
mod player_system;
mod process_update_command_system;
mod render_system;
mod world;

use enemy_system::enemy_system;
use input_system::input_system;
use late_update_system::late_update_system;
use player_system::player_system;
use process_update_command_system::process_update_command_system;
use render_system::render_system;
use world::World;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Phase {
    Input,
    UpdateSubmitCommand,
    UpdateProcessCommand,
    LateUpdate,
    Render,
}

fn main() {
    let world = Arc::new(RwLock::new(World::new()));
    let (sender, receiver) = channel();
    let stdout = Arc::new(Mutex::new(stdout()));

    let mut runtime = Runtime::new();

    runtime.activate_phase(Phase::Input, 0);
    runtime.activate_phase(Phase::UpdateSubmitCommand, 10);
    runtime.activate_phase(Phase::UpdateProcessCommand, 11);
    runtime.activate_phase(Phase::LateUpdate, 30);
    runtime.activate_phase(Phase::Render, 40);

    runtime.spawn(Phase::Input, input_system(Arc::clone(&world)));

    runtime.spawn(
        Phase::UpdateSubmitCommand,
        player_system(Arc::clone(&world), sender.clone()),
    );
    runtime.spawn(
        Phase::UpdateSubmitCommand,
        enemy_system(Arc::clone(&world), sender.clone()),
    );

    runtime.spawn(
        Phase::UpdateProcessCommand,
        process_update_command_system(Arc::clone(&world), receiver),
    );

    runtime.spawn(Phase::LateUpdate, late_update_system(Arc::clone(&world)));

    runtime.spawn(Phase::Render, render_system(Arc::clone(&world), stdout));

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
