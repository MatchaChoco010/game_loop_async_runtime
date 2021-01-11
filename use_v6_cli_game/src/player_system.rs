#![allow(unused_must_use)]

use std::sync::mpsc::Sender;

use runtime_v6::{next_frame, Read, Runtime};

use crate::world::{Direction, GameCommand, GameWorld, PlayerCommand, HEIGHT, WIDTH};
use crate::Phase;

pub async fn player_system(
    world: Read<GameWorld>,
    sender: Sender<GameCommand>,
    _runtime: Runtime<Phase, GameWorld>,
) {
    'update_loop: loop {
        if world.input.left {
            if world.player.x > 0 {
                sender.send(GameCommand::Player(PlayerCommand::Move(Direction::Left)));
            }
            sender.send(GameCommand::Player(PlayerCommand::SetDir(Direction::Left)));
        } else if world.input.right {
            if world.player.x < WIDTH - 1 {
                sender.send(GameCommand::Player(PlayerCommand::Move(Direction::Right)));
            }
            sender.send(GameCommand::Player(PlayerCommand::SetDir(Direction::Right)));
        } else if world.input.up {
            if world.player.y > 0 {
                sender.send(GameCommand::Player(PlayerCommand::Move(Direction::Up)));
            }
            sender.send(GameCommand::Player(PlayerCommand::SetDir(Direction::Up)));
        } else if world.input.down {
            if world.player.y < HEIGHT - 1 {
                sender.send(GameCommand::Player(PlayerCommand::Move(Direction::Down)));
            }
            sender.send(GameCommand::Player(PlayerCommand::SetDir(Direction::Down)));
        }

        if world.input.z {
            sender.send(GameCommand::Player(PlayerCommand::SetAttacked(true)));
        } else {
            sender.send(GameCommand::Player(PlayerCommand::SetAttacked(false)));
        }

        if world.should_stop_game {
            break 'update_loop;
        }

        next_frame().await;
    }
}
