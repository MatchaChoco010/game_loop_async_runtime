use rand::prelude::*;

use runtime_v6::World;

pub const WIDTH: u16 = 30;
pub const HEIGHT: u16 = 20;

pub enum InputCommand {
    Reset,
    Up,
    Down,
    Left,
    Right,
    Z,
}
pub enum PlayerCommand {
    Move(Direction),
    SetDir(Direction),
    SetAttacked(bool),
    Dead,
}

pub enum EnemyCommand {
    Move(usize, Direction),
    Kill(usize),
}

pub enum WorldStateCommand {
    SetGameOver,
    SetGameClear,
}

pub enum GameCommand {
    Input(InputCommand),
    Player(PlayerCommand),
    Enemy(EnemyCommand),
    WorldState(WorldStateCommand),
    ShouldStopGame,
}

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

#[derive(Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

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

#[derive(Clone, Copy)]
pub enum GameState {
    InGame,
    GameOver,
    GameClear,
}

pub struct GameWorld {
    pub state: GameState,
    pub should_stop_game: bool,
    pub input: Input,
    pub player: Player,
    pub enemies: Vec<Enemy>,
}
impl GameWorld {
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
impl World for GameWorld {
    type Command = GameCommand;
    fn process_command(&mut self, cmd: Self::Command) {
        match cmd {
            GameCommand::Input(input) => match input {
                InputCommand::Reset => self.input.reset(),
                InputCommand::Left => self.input.left = true,
                InputCommand::Right => self.input.right = true,
                InputCommand::Up => self.input.up = true,
                InputCommand::Down => self.input.down = true,
                InputCommand::Z => self.input.z = true,
            },
            GameCommand::Player(cmd) => match cmd {
                PlayerCommand::Move(dir) => match dir {
                    Direction::Left => self.player.x -= 1,
                    Direction::Right => self.player.x += 1,
                    Direction::Up => self.player.y -= 1,
                    Direction::Down => self.player.y += 1,
                },
                PlayerCommand::SetAttacked(flag) => self.player.attacked = flag,
                PlayerCommand::SetDir(dir) => self.player.dir = dir,
                PlayerCommand::Dead => self.player.dead = true,
            },
            GameCommand::Enemy(cmd) => match cmd {
                EnemyCommand::Move(index, dir) => match dir {
                    Direction::Left => self.enemies[index].x -= 1,
                    Direction::Right => self.enemies[index].x += 1,
                    Direction::Up => self.enemies[index].y -= 1,
                    Direction::Down => self.enemies[index].y += 1,
                },
                EnemyCommand::Kill(index) => self.enemies[index].dead = true,
            },
            GameCommand::WorldState(state) => match state {
                WorldStateCommand::SetGameOver => self.state = GameState::GameOver,
                WorldStateCommand::SetGameClear => self.state = GameState::GameClear,
            },
            GameCommand::ShouldStopGame => self.should_stop_game = true,
        }
    }
}
