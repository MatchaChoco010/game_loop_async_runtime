use std::thread::{self, sleep};
use std::time::{Duration, Instant};

use futures::join;

use runtime_v5::{next_frame, Runtime, RuntimeIsDone};

async fn pre_task() {
    for i in 0..10 {
        println!("-------- Frame {} start --------", i);
        println!("Pre Task: {}, ThreadID: [{:?}]", i, thread::current().id());
        next_frame().await;
    }
}

async fn task_1_1() {
    for i in 0..5 {
        println!("Task 1-1: {}, ThreadID: [{:?}]", i, thread::current().id());
        next_frame().await;
    }
}

async fn task_1_2() {
    for i in 0..5 {
        println!("Task 1-2: {}, ThreadID: [{:?}]", i, thread::current().id());
        next_frame().await;
    }
}

async fn task_1() {
    for i in 0..5 {
        println!("Task 1: {}, ThreadID: [{:?}]", i, thread::current().id());
        next_frame().await;
    }
    join!(task_1_1(), task_1_2());
}

async fn task_2() {
    for i in 0..10 {
        println!("Task 2: {}, ThreadID: [{:?}]", i, thread::current().id());
        next_frame().await;
    }
}

async fn task_3_1() {
    for i in 0..5 {
        println!("Task 3-1: {}, ThreadID: [{:?}]", i, thread::current().id());
        next_frame().await;
    }
}

async fn task_3_2() {
    for i in 0..5 {
        println!("Task 3-2: {}, ThreadID: [{:?}]", i, thread::current().id());
        next_frame().await;
    }
}

async fn task_3() {
    join!(task_3_1(), task_3_2());
    for i in 0..5 {
        println!("Task 3: {}, ThreadID: [{:?}]", i, thread::current().id());
        next_frame().await;
    }
}

async fn post_task_1() {
    for i in 0..5 {
        println!(
            "Post Task 1: {}, ThreadID: [{:?}]",
            i,
            thread::current().id()
        );
        next_frame().await;
    }
}

async fn post_task_2() {
    for i in 0..5 {
        println!(
            "Post Task 2: {}, ThreadID: [{:?}]",
            i,
            thread::current().id()
        );
        next_frame().await;
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Phase {
    PreTask,
    Task,
    PostTask,
}

fn main() {
    let mut runtime = Runtime::new();

    runtime.activate_phase(Phase::PreTask, 0);
    runtime.activate_phase(Phase::Task, 10);
    runtime.activate_phase(Phase::PostTask, 20);

    runtime.spawn(Phase::PreTask, pre_task());
    runtime.spawn(Phase::Task, task_1());
    runtime.spawn(Phase::Task, task_2());
    runtime.spawn(Phase::Task, task_3());
    runtime.spawn(Phase::PostTask, post_task_1());
    runtime.spawn(Phase::PostTask, post_task_2());

    'update_loop: loop {
        let frame_start = Instant::now();
        let frame_duration = Duration::new(1, 0);

        match runtime.update() {
            RuntimeIsDone::Done => break 'update_loop,
            RuntimeIsDone::NotDone => (),
        }

        let now = Instant::now();
        let duration = now.duration_since(frame_start);
        if duration < frame_duration {
            sleep(frame_duration - duration);
        }
    }
}
