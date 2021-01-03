use std::{future::Future, pin::Pin, task::Context};

use futures::task;

pub enum RuntimeIsDone {
    Done,
    NotDone,
}

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

    pub fn update(&mut self) -> RuntimeIsDone {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);

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
            return RuntimeIsDone::Done;
        }

        self.current_pool_index = (self.current_pool_index + 1) % 2;
        self.frame_counter = self.frame_counter.wrapping_add(1);

        RuntimeIsDone::NotDone
    }

    pub const fn frame_counter(&self) -> u64 {
        self.frame_counter
    }
}
