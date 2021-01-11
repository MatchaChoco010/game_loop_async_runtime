#![allow(unused_must_use)]

use std::io::stdout;
use std::io::Write;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{future::FutureExt, pin_mut, select};

use runtime_v6::{next_frame, Read, Runtime};

use crate::world::{Direction, GameCommand, GameState, GameWorld, HEIGHT, WIDTH};
use crate::Phase;

const GAMEOVER0: &str = r"   _____          __  __ ______ ______      ________ _____   ";
const GAMEOVER1: &str = r"  / ____|   /\   |  \/  |  ____/ __ \ \    / /  ____|  __ \  ";
const GAMEOVER2: &str = r" | |  __   /  \  | \  / | |__ | |  | \ \  / /| |__  | |__) | ";
const GAMEOVER3: &str = r" | | |_ | / /\ \ | |\/| |  __|| |  | |\ \/ / |  __| |  _  /  ";
const GAMEOVER4: &str = r" | |__| |/ ____ \| |  | | |___| |__| | \  /  | |____| | \ \  ";
const GAMEOVER5: &str = r"  \_____/_/    \_\_|  |_|______\____/   \/   |______|_|  \_\ ";

const CLEAR0: &str = r"            _____ _      ______          _____               ";
const CLEAR1: &str = r"           / ____| |    |  ____|   /\   |  __ \              ";
const CLEAR2: &str = r"          | |    | |    | |__     /  \  | |__) |             ";
const CLEAR3: &str = r"          | |    | |    |  __|   / /\ \ |  _  /              ";
const CLEAR4: &str = r"          | |____| |____| |____ / ____ \| | \ \              ";
const CLEAR5: &str = r"           \_____|______|______/_/    \_\_|  \_\             ";

async fn game_over(sender: Sender<GameCommand>, w: Arc<Mutex<impl Write>>) {
    let size = terminal::size().unwrap();
    let offset_x = if size.0 / 2 < WIDTH + 2 {
        println!("Terminal space is too small!");
        0
    } else {
        size.0 / 2 - (WIDTH + 2)
    };
    let offset_y = if size.1 / 2 < HEIGHT / 2 + 2 {
        println!("Terminal space is too small!");
        0
    } else {
        size.1 / 2 - (HEIGHT / 2 + 2)
    };

    {
        let mut w = w.lock().expect("Get write");
        execute!(w, SetForegroundColor(Color::Magenta)).unwrap();
    }

    let scroll_width = WIDTH * 2;
    for i in 0..scroll_width {
        {
            let mut w = w.lock().expect("Get write");
            let space = WIDTH * 2 + 2 - i;
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 6),
                Clear(ClearType::CurrentLine),
                Print(GAMEOVER0)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 7),
                Clear(ClearType::CurrentLine),
                Print(GAMEOVER1)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 8),
                Clear(ClearType::CurrentLine),
                Print(GAMEOVER2)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 9),
                Clear(ClearType::CurrentLine),
                Print(GAMEOVER3)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 10),
                Clear(ClearType::CurrentLine),
                Print(GAMEOVER4)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 11),
                Clear(ClearType::CurrentLine),
                Print(GAMEOVER5)
            )
            .unwrap();
            queue!(w, MoveTo(1, 1)).unwrap();
            w.flush().unwrap();
        }

        next_frame().await;
    }

    for _ in 0..15 {
        next_frame().await;
    }

    sender.send(GameCommand::ShouldStopGame);
}

async fn game_clear(sender: Sender<GameCommand>, w: Arc<Mutex<impl Write>>) {
    let size = terminal::size().unwrap();
    let offset_x = if size.0 / 2 < WIDTH + 2 {
        println!("Terminal space is too small!");
        0
    } else {
        size.0 / 2 - (WIDTH + 2)
    };
    let offset_y = if size.1 / 2 < HEIGHT / 2 + 2 {
        println!("Terminal space is too small!");
        0
    } else {
        size.1 / 2 - (HEIGHT / 2 + 2)
    };

    {
        let mut w = w.lock().expect("Get write");
        execute!(w, SetForegroundColor(Color::Cyan)).unwrap();
    }

    let scroll_width = WIDTH * 2;
    for i in 0..scroll_width {
        {
            let mut w = w.lock().expect("Get write");
            let space = WIDTH * 2 + 2 - i;
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 6),
                Clear(ClearType::CurrentLine),
                Print(CLEAR0)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 7),
                Clear(ClearType::CurrentLine),
                Print(CLEAR1)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 8),
                Clear(ClearType::CurrentLine),
                Print(CLEAR2)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 9),
                Clear(ClearType::CurrentLine),
                Print(CLEAR3)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 10),
                Clear(ClearType::CurrentLine),
                Print(CLEAR4)
            )
            .unwrap();
            queue!(
                w,
                MoveTo(offset_x + space, offset_y + 11),
                Clear(ClearType::CurrentLine),
                Print(CLEAR5)
            )
            .unwrap();
            queue!(w, MoveTo(1, 1)).unwrap();
            w.flush().unwrap();
        }

        next_frame().await;
    }

    for _ in 0..15 {
        next_frame().await;
    }

    sender.send(GameCommand::ShouldStopGame);
}

async fn game_close(world: Read<GameWorld>) {
    'update_loop: loop {
        if world.should_stop_game {
            break 'update_loop;
        }

        next_frame().await;
    }
}

async fn render(world: Read<GameWorld>, sender: Sender<GameCommand>, w: Arc<Mutex<impl Write>>) {
    loop {
        let state = {
            let mut w = w.lock().expect("Get write");

            queue!(w, Clear(ClearType::All)).unwrap();

            let size = terminal::size().unwrap();
            let offset_x = if size.0 / 2 < WIDTH + 2 {
                println!("Terminal space is too small!");
                0
            } else {
                size.0 / 2 - (WIDTH + 2)
            };
            let offset_y = if size.1 / 2 < HEIGHT / 2 + 2 {
                println!("Terminal space is too small!");
                0
            } else {
                size.1 / 2 - (HEIGHT / 2 + 2)
            };

            // 枠
            {
                let mut top_bottom_row = "".to_string();
                for _ in 0..(WIDTH + 2) {
                    top_bottom_row.push_str("##");
                }
                let top_bottom_row = top_bottom_row.as_str();

                queue!(w, SetForegroundColor(Color::DarkGrey)).unwrap();
                queue!(
                    w,
                    MoveTo(offset_x + 2, offset_y + 2),
                    Print(&top_bottom_row)
                )
                .unwrap();
                for i in 0..HEIGHT {
                    queue!(
                        w,
                        MoveTo(offset_x + 2, offset_y + 2 + i + 1),
                        Print("##"),
                        MoveTo(offset_x + 2 + WIDTH * 2 + 2, offset_y + 2 + i + 1),
                        Print("##")
                    )
                    .unwrap();
                }
                queue!(
                    w,
                    MoveTo(offset_x + 2, offset_y + 2 + HEIGHT + 1),
                    Print(top_bottom_row)
                )
                .unwrap();
            }

            // Enemy
            {
                for e in world.enemies.iter() {
                    let x = offset_x + e.x * 2 + 2 + 2;
                    let y = offset_y + e.y + 2 + 1;
                    queue!(w, MoveTo(x, y)).unwrap();
                    if e.dead {
                        queue!(w, SetForegroundColor(Color::DarkRed), Print('血')).unwrap();
                    } else {
                        queue!(w, SetForegroundColor(Color::DarkCyan), Print('獣')).unwrap();
                    }
                }
            }

            // Player
            if !world.player.dead {
                let x = offset_x + world.player.x * 2 + 2 + 2;
                let y = offset_y + world.player.y + 2 + 1;
                queue!(w, MoveTo(x, y)).unwrap();
                queue!(w, SetForegroundColor(Color::DarkYellow), Print('人')).unwrap();

                queue!(w, SetForegroundColor(Color::White)).unwrap();
                if world.player.attacked {
                    match world.player.dir {
                        Direction::Left => queue!(w, MoveTo(x - 2, y), Print('刀')).unwrap(),
                        Direction::Right => queue!(w, MoveTo(x + 2, y), Print('刀')).unwrap(),
                        Direction::Up => queue!(w, MoveTo(x, y - 1), Print('刀')).unwrap(),
                        Direction::Down => queue!(w, MoveTo(x, y + 1), Print('刀')).unwrap(),
                    }
                }
            }

            queue!(w, MoveTo(1, 1)).unwrap();
            w.flush().unwrap();

            world.state
        };

        match state {
            GameState::InGame => next_frame().await,
            GameState::GameClear => {
                game_clear(sender, Arc::clone(&w)).await;
                break;
            }
            GameState::GameOver => {
                game_over(sender, Arc::clone(&w)).await;
                break;
            }
        }
    }
}

pub async fn render_system(
    world: Read<GameWorld>,
    sender: Sender<GameCommand>,
    _runtime: Runtime<Phase, GameWorld>,
) {
    let w = Arc::new(Mutex::new(stdout()));

    {
        let mut w = w.lock().expect("Get write");
        execute!(w, EnterAlternateScreen).unwrap();
    }

    let render = render(world, sender.clone(), Arc::clone(&w)).fuse();
    let close = game_close(world).fuse();
    pin_mut!(render);
    pin_mut!(close);
    select! {
        _ = render => (),
        _ = close => (),
    }

    {
        let mut w = w.lock().expect("Get write");
        execute!(w, LeaveAlternateScreen).unwrap();
    }
}
