use std::{
    future::Future,
    pin::Pin,
    task::Context,
    thread::sleep,
    time::{Duration, Instant},
};

use futures::task;

type Task = Pin<Box<dyn Future<Output = ()> + 'static>>;

pub struct Runtime {
    frame_counter: u64,
    current_pool_index: u8,
    task_pool_0: Vec<Task>,
    task_pool_1: Vec<Task>,
}
impl Runtime {
    pub fn new() -> Self {
        Self {
            frame_counter: 0,
            current_pool_index: 0,
            task_pool_0: vec![],
            task_pool_1: vec![],
        }
    }

    pub fn spawn(&mut self, f: impl Future<Output = ()> + 'static) {
        if self.current_pool_index == 0 {
            self.task_pool_0.push(Box::pin(f));
        } else {
            self.task_pool_1.push(Box::pin(f));
        }
    }

    pub fn run(&mut self) {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let frame_duration = Duration::new(0, 16_666_666);

        loop {
            let frame_start = Instant::now();

            let (current_task_pool, next_frame_task_pool) = if self.current_pool_index == 0 {
                (&mut self.task_pool_0, &mut self.task_pool_1)
            } else {
                (&mut self.task_pool_1, &mut self.task_pool_0)
            };

            while let Some(mut task) = current_task_pool.pop() {
                if task.as_mut().poll(&mut cx).is_pending() {
                    next_frame_task_pool.push(task);
                }
            }

            if next_frame_task_pool.len() == 0 {
                return;
            }

            let now = Instant::now();
            let duration = now.duration_since(frame_start);
            if duration < frame_duration {
                sleep(frame_duration - duration);
            }

            self.current_pool_index = (self.current_pool_index + 1) % 2;
            self.frame_counter = self.frame_counter.wrapping_add(1);
        }
    }

    pub const fn frame_counter(&self) -> u64 {
        self.frame_counter
    }
}
