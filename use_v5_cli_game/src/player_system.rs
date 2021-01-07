use std::sync::{Arc, Mutex};

use runtime_v5::next_frame;

use crate::world::{Direction, World, HEIGHT, WIDTH};

pub async fn player_system(world: Arc<Mutex<World>>) {
    'update_loop: loop {
        {
            let mut world = world.lock().unwrap();
            if world.input.left {
                if world.player.x > 0 {
                    world.player.x -= 1;
                }
                world.player.dir = Direction::Left;
            } else if world.input.right {
                if world.player.x < WIDTH - 1 {
                    world.player.x += 1;
                }
                world.player.dir = Direction::Right;
            } else if world.input.up {
                if world.player.y > 0 {
                    world.player.y -= 1;
                }
                world.player.dir = Direction::Up;
            } else if world.input.down {
                if world.player.y < HEIGHT - 1 {
                    world.player.y += 1;
                }
                world.player.dir = Direction::Down;
            }

            if world.input.z {
                world.player.attacked = true;
            } else {
                world.player.attacked = false;
            }

            if world.should_stop_game {
                break 'update_loop;
            }
        }

        next_frame().await;
    }
}
