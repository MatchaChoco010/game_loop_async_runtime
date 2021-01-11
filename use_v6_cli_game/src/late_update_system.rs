#![allow(unused_must_use)]

use std::sync::mpsc::Sender;

use runtime_v6::{next_frame, Read, Runtime};

use crate::world::{
    Direction, EnemyCommand, GameCommand, GameState, GameWorld, PlayerCommand, WorldStateCommand,
};
use crate::Phase;

pub async fn late_update_system(
    world: Read<GameWorld>,
    sender: Sender<GameCommand>,
    _runtime: Runtime<Phase, GameWorld>,
) {
    'update_loop: loop {
        match world.state {
            GameState::GameClear => (),
            GameState::GameOver => (),
            GameState::InGame => {
                // Enemy
                {
                    if world.player.attacked {
                        let x = world.player.x as i16;
                        let y = world.player.y as i16;
                        let (x, y) = match world.player.dir {
                            Direction::Left => (x - 1, y),
                            Direction::Right => (x + 1, y),
                            Direction::Up => (x, y - 1),
                            Direction::Down => (x, y + 1),
                        };
                        for (index, e) in world.enemies.iter().enumerate() {
                            if e.x as i16 == x && e.y as i16 == y {
                                sender.send(GameCommand::Enemy(EnemyCommand::Kill(index)));
                            }
                        }
                    }
                }

                // Player
                {
                    let mut dead = false;
                    for e in world.enemies.iter() {
                        if e.dead {
                            continue;
                        }
                        if world.player.x == e.x && world.player.y == e.y {
                            dead = true;
                        }
                    }
                    if dead {
                        sender.send(GameCommand::Player(PlayerCommand::Dead));
                    }
                }

                // 終了処理
                {
                    if world.player.dead {
                        sender.send(GameCommand::WorldState(WorldStateCommand::SetGameOver));
                    } else if world.enemies.iter().all(|e| e.dead) {
                        sender.send(GameCommand::WorldState(WorldStateCommand::SetGameClear));
                    }
                }
            }
        }

        if world.should_stop_game {
            break 'update_loop;
        }

        next_frame().await;
    }
}
