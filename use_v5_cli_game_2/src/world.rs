use rand::prelude::*;

pub const WIDTH: u16 = 30;
pub const HEIGHT: u16 = 20;

#[derive(Debug)]
pub struct Input {
    pub z: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}
impl Input {
    fn new() -> Self {
        Self {
            z: false,
            left: false,
            right: false,
            up: false,
            down: false,
        }
    }

    pub fn reset(&mut self) {
        self.z = false;
        self.left = false;
        self.right = false;
        self.up = false;
        self.down = false;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug)]
pub struct Player {
    pub dead: bool,
    pub x: u16,
    pub y: u16,
    pub dir: Direction,
    pub attacked: bool,
}
impl Player {
    fn new(x: u16, y: u16) -> Self {
        Self {
            dead: false,
            x,
            y,
            dir: Direction::Right,
            attacked: false,
        }
    }
}

#[derive(Debug)]
pub struct Enemy {
    pub dead: bool,
    pub x: u16,
    pub y: u16,
}
impl Enemy {
    fn new(x: u16, y: u16) -> Self {
        Self { dead: false, x, y }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    InGame,
    GameOver,
    GameClear,
}

#[derive(Debug)]
pub struct World {
    pub state: GameState,
    pub should_stop_game: bool,
    pub input: Input,
    pub player: Player,
    pub enemies: Vec<Enemy>,
}
impl World {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        let mut enemies: Vec<Enemy> = vec![];
        for _ in 0..5 {
            let (x, y) = 'label: loop {
                let x = rng.gen_range(0..WIDTH);
                let y = rng.gen_range(0..HEIGHT);

                if x > 4 || y > 4 {
                    if enemies.is_empty() {
                        break 'label (x, y);
                    } else {
                        for e in enemies.iter() {
                            if e.x != x || e.y != y {
                                break 'label (x, y);
                            }
                        }
                    }
                }
            };
            enemies.push(Enemy::new(x, y));
        }

        Self {
            state: GameState::InGame,
            should_stop_game: false,
            input: Input::new(),
            player: Player::new(2, 2),
            enemies,
        }
    }
}

pub enum Command {
    MovePlayer(Direction),
    SetPlayerDir(Direction),
    SetPlayerAttacked(bool),
    MoveEnemy(usize, Direction),
}
