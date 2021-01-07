use std::sync::{Arc, Mutex};

use rand::prelude::*;

use runtime_v5::next_frame;

use crate::world::{World, HEIGHT, WIDTH};

pub async fn enemy_system(world: Arc<Mutex<World>>) {
    let mut i = 0;

    'update_loop: loop {
        if i % 8 == 0 {
            let mut world = world.lock().unwrap();
            let mut rng = rand::thread_rng();

            for e in world.enemies.iter_mut() {
                if e.dead {
                    continue;
                }

                let dir = rng.gen_range(0..4);
                match dir {
                    0 => {
                        if e.x > 0 {
                            e.x -= 1;
                        }
                    }
                    1 => {
                        if e.x < WIDTH - 1 {
                            e.x += 1;
                        }
                    }
                    2 => {
                        if e.y > 0 {
                            e.y -= 1;
                        }
                    }
                    3 => {
                        if e.y < HEIGHT - 1 {
                            e.y += 1;
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
