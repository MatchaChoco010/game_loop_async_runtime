use std::sync::{mpsc::Receiver, Arc, RwLock};

use runtime_v5::next_frame;

use crate::world::{Command, Direction, World};

pub async fn process_update_command_system(world: Arc<RwLock<World>>, receiver: Receiver<Command>) {
    'update_loop: loop {
        {
            let mut world = world.write().unwrap();

            for cmd in receiver.try_iter() {
                match cmd {
                    Command::MovePlayer(dir) => match dir {
                        Direction::Left => world.player.x -= 1,
                        Direction::Right => world.player.x += 1,
                        Direction::Up => world.player.y -= 1,
                        Direction::Down => world.player.y += 1,
                    },
                    Command::MoveEnemy(index, dir) => match dir {
                        Direction::Left => world.enemies[index].x -= 1,
                        Direction::Right => world.enemies[index].x += 1,
                        Direction::Up => world.enemies[index].y -= 1,
                        Direction::Down => world.enemies[index].y += 1,
                    },
                    Command::SetPlayerAttacked(flag) => world.player.attacked = flag,
                    Command::SetPlayerDir(dir) => world.player.dir = dir,
                }
            }

            if world.should_stop_game {
                break 'update_loop;
            }
        }

        next_frame().await;
    }
}
