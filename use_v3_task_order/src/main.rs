use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use runtime_v3::{next_frame, Runtime, RuntimeIsDone};

async fn task_1() {
    for i in 0..5 {
        println!("Task 1: {}", i);
        next_frame().await;
    }
}

async fn task_2() {
    for i in 0..5 {
        println!("Task 2: {}", i);
        next_frame().await;
    }
}

async fn task_3() {
    for i in 0..5 {
        println!("Task 3: {}", i);
        next_frame().await;
    }
}

fn main() {
    let mut runtime = Runtime::new();

    runtime.spawn(task_1());
    runtime.spawn(task_2());
    runtime.spawn(task_3());

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
