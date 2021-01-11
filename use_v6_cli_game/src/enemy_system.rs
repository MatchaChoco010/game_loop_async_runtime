#![allow(unused_must_use)]

use std::sync::mpsc::Sender;

use rand::prelude::*;

use runtime_v6::{next_frame, Read, Runtime};

use crate::world::{Direction, EnemyCommand, GameCommand, GameWorld, HEIGHT, WIDTH};
use crate::Phase;

pub async fn enemy_system(
    world: Read<GameWorld>,
    sender: Sender<GameCommand>,
    _runtime: Runtime<Phase, GameWorld>,
) {
    let mut i = 0;

    'update_loop: loop {
        if i % 8 == 0 {
            let mut rng = rand::thread_rng();

            for (index, e) in world.enemies.iter().enumerate() {
                if e.dead {
                    continue;
                }

                let dir = rng.gen_range(0..4);
                match dir {
                    0 => {
                        if e.x > 0 {
                            sender.send(GameCommand::Enemy(EnemyCommand::Move(
                                index,
                                Direction::Left,
                            )));
                        }
                    }
                    1 => {
                        if e.x < WIDTH - 1 {
                            sender.send(GameCommand::Enemy(EnemyCommand::Move(
                                index,
                                Direction::Right,
                            )));
                        }
                    }
                    2 => {
                        if e.y > 0 {
                            sender
                                .send(GameCommand::Enemy(EnemyCommand::Move(index, Direction::Up)));
                        }
                    }
                    3 => {
                        if e.y < HEIGHT - 1 {
                            sender.send(GameCommand::Enemy(EnemyCommand::Move(
                                index,
                                Direction::Down,
                            )));
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        if world.should_stop_game {
            break 'update_loop;
        }

        i += 1;
        next_frame().await;
    }
}
