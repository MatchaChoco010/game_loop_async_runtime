#![allow(unused_must_use)]

use std::sync::mpsc::Sender;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use runtime_v6::{next_frame, Read, Runtime};

use crate::key_events::KeyEvents;
use crate::world::{GameCommand, GameWorld, InputCommand};
use crate::Phase;

pub async fn input_system(
    world: Read<GameWorld>,
    sender: Sender<GameCommand>,
    _runtime: Runtime<Phase, GameWorld>,
) {
    let mut key_events = KeyEvents::new();

    'update_loop: loop {
        sender.send(GameCommand::Input(InputCommand::Reset));
        for evt in key_events.get_events() {
            match evt {
                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                } => {
                    sender.send(GameCommand::ShouldStopGame);
                    break 'update_loop;
                }
                KeyEvent {
                    code: KeyCode::Char('z'),
                    modifiers: KeyModifiers::NONE,
                } => {
                    sender.send(GameCommand::Input(InputCommand::Z));
                }
                KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                } => {
                    sender.send(GameCommand::Input(InputCommand::Left));
                }
                KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                } => {
                    sender.send(GameCommand::Input(InputCommand::Right));
                }
                KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                } => {
                    sender.send(GameCommand::Input(InputCommand::Up));
                }
                KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                } => {
                    sender.send(GameCommand::Input(InputCommand::Down));
                }
                _ => (),
            }
        }

        if world.should_stop_game {
            break 'update_loop;
        }

        next_frame().await;
    }
}
