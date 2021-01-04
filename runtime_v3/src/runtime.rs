use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::Context;
use std::task::Waker;
use std::{future::Future, task::Poll};

use futures::task::ArcWake;

struct Task {
    future: Pin<Box<dyn Future<Output = ()> + 'static>>,
}
impl Task {
    fn new(f: impl Future<Output = ()> + 'static) -> Self {
        Self {
            future: Box::pin(f),
        }
    }

    fn poll(&mut self, mut ctx: Context) -> Poll<()> {
        match Future::poll(self.future.as_mut(), &mut ctx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(()) => Poll::Ready(()),
        }
    }
}

#[derive(Clone)]
struct WakeFlag {
    waked: Arc<Mutex<bool>>,
}
impl WakeFlag {
    fn new() -> Self {
        Self {
            waked: Arc::new(Mutex::new(false)),
        }
    }

    fn wake(&self) {
        *self.waked.lock().unwrap() = true;
    }

    fn is_waked(&self) -> bool {
        *self.waked.lock().unwrap()
    }
}

#[derive(Clone)]
struct WakeFlagWaker {
    flag: WakeFlag,
}
impl WakeFlagWaker {
    fn waker(flag: WakeFlag) -> Waker {
        futures::task::waker(Arc::new(Self { flag }))
    }
}
impl ArcWake for WakeFlagWaker {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.flag.wake();
    }
}

pub enum RuntimeIsDone {
    Done,
    NotDone,
}

#[derive(Clone)]
pub struct Runtime {
    frame_counter: u64,
    tasks_queue: Rc<RefCell<Vec<Task>>>,
    wait_tasks: Rc<RefCell<Vec<Task>>>,
}
impl Runtime {
    pub fn new() -> Self {
        Self {
            frame_counter: 0,
            tasks_queue: Rc::new(RefCell::new(vec![])),
            wait_tasks: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn spawn(&self, f: impl Future<Output = ()> + 'static) {
        self.tasks_queue.borrow_mut().push(Task::new(f));
    }

    pub fn update(&mut self) -> RuntimeIsDone {
        'current_frame: loop {
            let task = self.tasks_queue.borrow_mut().pop();

            match task {
                // task_queueが空だった場合はループを抜ける
                None => break 'current_frame,
                Some(mut task) => {
                    let flag = WakeFlag::new();
                    let waker = WakeFlagWaker::waker(flag.clone());

                    match task.poll(Context::from_waker(&waker)) {
                        Poll::Ready(()) => (),
                        Poll::Pending => {
                            // タスクがwake済みだったらtask_queueにpush
                            // そうでなかったらwait_tasksにpushする
                            if flag.is_waked() {
                                self.tasks_queue.borrow_mut().push(task);
                            } else {
                                self.wait_tasks.borrow_mut().push(task);
                            }
                        }
                    }
                }
            }
        }

        // wait_tasksが空の場合、全てのタスクの実行が終わっている。
        if self.wait_tasks.borrow().is_empty() {
            return RuntimeIsDone::Done;
        }

        // 次のフレームに移る前にフレームカウンターを更新する
        self.frame_counter += 1;

        // wait_tasksを空のtasks_queueと交換する
        std::mem::swap(&mut self.wait_tasks, &mut self.tasks_queue);

        RuntimeIsDone::NotDone
    }

    pub fn frame_counter(&self) -> u64 {
        self.frame_counter
    }
}
