#![allow(unused_must_use)]

use std::sync::{mpsc::Sender, Arc, RwLock};

use rand::prelude::*;

use runtime_v5::next_frame;

use crate::world::{Command, Direction, World, HEIGHT, WIDTH};

pub async fn enemy_system(world: Arc<RwLock<World>>, sender: Sender<Command>) {
    let mut i = 0;

    'update_loop: loop {
        if i % 8 == 0 {
            let world = world.read().unwrap();
            let mut rng = rand::thread_rng();

            for (index, e) in world.enemies.iter().enumerate() {
                if e.dead {
                    continue;
                }

                let dir = rng.gen_range(0..4);
                match dir {
                    0 => {
                        if e.x > 0 {
                            sender.send(Command::MoveEnemy(index, Direction::Left));
                        }
                    }
                    1 => {
                        if e.x < WIDTH - 1 {
                            sender.send(Command::MoveEnemy(index, Direction::Right));
                        }
                    }
                    2 => {
                        if e.y > 0 {
                            sender.send(Command::MoveEnemy(index, Direction::Up));
                        }
                    }
                    3 => {
                        if e.y < HEIGHT - 1 {
                            sender.send(Command::MoveEnemy(index, Direction::Down));
                        }
                    }
                    _ => unreachable!(),
                }
            }

            if world.should_stop_game {
                break 'update_loop;
            }
        }

        i += 1;
        next_frame().await;
    }
}
