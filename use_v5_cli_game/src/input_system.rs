use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use runtime_v5::next_frame;

use crate::key_events::KeyEvents;
use crate::world::World;

pub async fn input_system(world: Arc<Mutex<World>>) {
    let mut key_events = KeyEvents::new();

    'update_loop: loop {
        {
            let mut world = world.lock().expect("Get world");
            world.input.reset();

            for evt in key_events.get_events() {
                match evt {
                    KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    } => world.should_stop_game = true,
                    KeyEvent {
                        code: KeyCode::Char('z'),
                        modifiers: KeyModifiers::NONE,
                    } => world.input.z = true,
                    KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    } => world.input.left = true,
                    KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    } => world.input.right = true,
                    KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    } => world.input.up = true,
                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    } => world.input.down = true,
                    _ => (),
                }
            }

            if world.should_stop_game {
                break 'update_loop;
            }
        }

        next_frame().await;
    }
}
