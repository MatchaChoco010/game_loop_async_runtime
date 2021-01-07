use std::sync::{Arc, Mutex};

use runtime_v5::next_frame;

use crate::world::{Direction, GameState, World};

pub async fn late_update_system(world: Arc<Mutex<World>>) {
    'update_loop: loop {
        {
            let mut world = world.lock().unwrap();

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
                            for e in world.enemies.iter_mut() {
                                if e.x as i16 == x && e.y as i16 == y {
                                    e.dead = true;
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
                            world.player.dead = true;
                        }
                    }

                    // 終了処理
                    {
                        if world.player.dead {
                            world.state = GameState::GameOver;
                        } else if world.enemies.iter().all(|e| e.dead) {
                            world.state = GameState::GameClear;
                        }
                    }
                }
            }

            if world.should_stop_game {
                break 'update_loop;
            }
        }

        next_frame().await;
    }
}
