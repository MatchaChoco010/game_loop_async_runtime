use runtime_v6::*;
use std::sync::mpsc::Sender;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Phase {
    Input,
    Update,
}

enum MyWorldCommand {
    Add(f64),
    Sub(f64),
}

struct MyWorld {
    field: f64,
}
impl World for MyWorld {
    type Command = MyWorldCommand;
    fn process_command(&mut self, cmd: Self::Command) {
        match cmd {
            MyWorldCommand::Add(f) => self.field += f,
            MyWorldCommand::Sub(f) => self.field -= f,
        }
    }
}

async fn hey(
    world: Read<MyWorld>,
    sender: Sender<MyWorldCommand>,
    _runtime: Runtime<Phase, MyWorld>,
) {
    let f = world.field;
    println!("{}", f);

    sender.send(MyWorldCommand::Add(10.0)).unwrap();
    next_frame().await;

    let f = world.field;
    println!("{}", f);

    sender.send(MyWorldCommand::Sub(10.0)).unwrap();
    next_frame().await;

    let f = world.field;
    println!("{}", f);
}

async fn update2(
    _world: Read<MyWorld>,
    _sender: Sender<MyWorldCommand>,
    _runtime: Runtime<Phase, MyWorld>,
) {
    for i in 0..5 {
        println!("update2: {}", i);
        next_frame().await;
    }
}

async fn update(
    _world: Read<MyWorld>,
    _sender: Sender<MyWorldCommand>,
    runtime: Runtime<Phase, MyWorld>,
) {
    println!("update: {}", 0);
    next_frame().await;
    println!("update: {}", 1);
    next_frame().await;

    runtime.add_async_system(Phase::Update, update2);
    next_frame().await;

    for i in 2..10 {
        println!("update: {}", i);
        next_frame().await;
    }
}

fn main() {
    let world = MyWorld { field: 0.0 };

    let mut runtime = Runtime::<Phase, MyWorld>::new(world);

    runtime.activate_phase(Phase::Input, 0);
    runtime.activate_phase(Phase::Update, 10);

    runtime.add_async_system(Phase::Input, hey);
    runtime.add_async_system(Phase::Update, update);

    'update_loop: loop {
        match runtime.update() {
            RuntimeIsDone::Done => break 'update_loop,
            RuntimeIsDone::NotDone => (),
        }
    }
}
