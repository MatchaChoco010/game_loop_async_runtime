#![allow(unused_must_use)]

use std::sync::{mpsc::Sender, Arc, RwLock};

use runtime_v5::next_frame;

use crate::world::{Command, Direction, World, HEIGHT, WIDTH};

pub async fn player_system(world: Arc<RwLock<World>>, sender: Sender<Command>) {
    'update_loop: loop {
        {
            let world = world.read().unwrap();
            if world.input.left {
                if world.player.x > 0 {
                    sender.send(Command::MovePlayer(Direction::Left));
                }
                sender.send(Command::SetPlayerDir(Direction::Left));
            } else if world.input.right {
                if world.player.x < WIDTH - 1 {
                    sender.send(Command::MovePlayer(Direction::Right));
                }
                sender.send(Command::SetPlayerDir(Direction::Right));
            } else if world.input.up {
                if world.player.y > 0 {
                    sender.send(Command::MovePlayer(Direction::Up));
                }
                sender.send(Command::SetPlayerDir(Direction::Up));
            } else if world.input.down {
                if world.player.y < HEIGHT - 1 {
                    sender.send(Command::MovePlayer(Direction::Down));
                }
                sender.send(Command::SetPlayerDir(Direction::Down));
            }

            if world.input.z {
                sender.send(Command::SetPlayerAttacked(true));
            } else {
                sender.send(Command::SetPlayerAttacked(false));
            }

            if world.should_stop_game {
                break 'update_loop;
            }
        }

        next_frame().await;
    }
}
