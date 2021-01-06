use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread::{self, JoinHandle};
use std::{cell::RefCell, time::Duration};

use futures::task::ArcWake;

struct Task {
    future: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
}
impl Task {
    fn new(f: impl Future<Output = ()> + Send + 'static) -> Self {
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

/// 非同期タスクがすべて終了したかどうかのenum。
pub enum RuntimeIsDone {
    Done,
    NotDone,
}

fn process_tasks(mut tasks: Vec<Task>) -> Vec<Task> {
    let mut wait_tasks = vec![];

    'current_frame: loop {
        let task = tasks.pop();

        match task {
            // tasksが空だった場合は次のphaseへ
            None => break 'current_frame,
            Some(mut task) => {
                let flag = WakeFlag::new();
                let waker = WakeFlagWaker::waker(flag.clone());

                match task.poll(Context::from_waker(&waker)) {
                    Poll::Ready(()) => (),
                    Poll::Pending => {
                        // タスクがwake済みだったらtasksにpush
                        // そうでなかったらwait_tasksにpushする
                        if flag.is_waked() {
                            tasks.push(task);
                        } else {
                            wait_tasks.push(task);
                        }
                    }
                }
            }
        }
    }

    wait_tasks
}

/// ゲームループ用の非同期ランタイム。
#[derive(Clone)]
pub struct Runtime<T: Eq + Hash + Clone + Debug> {
    frame_counter: u64,
    tasks: Rc<RefCell<HashMap<T, Vec<Task>>>>,
    wait_tasks: Rc<RefCell<HashMap<T, Vec<Task>>>>,
    activated_phase: Rc<RefCell<HashMap<u16, T>>>,
    threads: Rc<RefCell<Vec<Option<JoinHandle<()>>>>>,
    receivers: Rc<[Receiver<Vec<Task>>; 2]>,
    senders: [Sender<Vec<Task>>; 2],
    thread_stop_flag: Arc<AtomicBool>,
}
impl<T: Eq + Hash + Clone + Debug> Runtime<T> {
    /// 新しくRuntimeを作成して返す。
    pub fn new() -> Self {
        let thread_stop_flag = Arc::new(AtomicBool::new(false));

        let (thread_sender1, main_receiver1) = channel();
        let (main_sender1, thread_receiver1) = channel();
        let stop_flag_thread1 = Arc::clone(&thread_stop_flag);
        let thread1 = thread::spawn(move || loop {
            match thread_receiver1.recv_timeout(Duration::from_millis(16)) {
                Ok(tasks) => {
                    let wait_tasks = process_tasks(tasks);
                    thread_sender1.send(wait_tasks).unwrap();
                }
                Err(_) => (),
            }

            if stop_flag_thread1.load(Ordering::Relaxed) {
                break;
            }
        });

        let (thread_sender2, main_receiver2) = channel();
        let (main_sender2, thread_receiver2) = channel();
        let stop_flag_thread2 = Arc::clone(&thread_stop_flag);
        let thread2 = thread::spawn(move || loop {
            match thread_receiver2.recv_timeout(Duration::from_millis(16)) {
                Ok(tasks) => {
                    let wait_tasks = process_tasks(tasks);
                    thread_sender2.send(wait_tasks).unwrap();
                }
                Err(_) => (),
            }

            if stop_flag_thread2.load(Ordering::Relaxed) {
                break;
            }
        });

        Self {
            frame_counter: 0,
            tasks: Rc::new(RefCell::new(HashMap::new())),
            wait_tasks: Rc::new(RefCell::new(HashMap::new())),
            activated_phase: Rc::new(RefCell::new(HashMap::new())),
            threads: Rc::new(RefCell::new(vec![Some(thread1), Some(thread2)])),
            receivers: Rc::new([main_receiver1, main_receiver2]),
            senders: [main_sender1, main_sender2],
            thread_stop_flag,
        }
    }

    /// タスクを起動する関数。
    /// 同一Phaseのタスクの実行順序は不定。
    pub fn spawn(&self, phase: T, f: impl Future<Output = ()> + Send + 'static) {
        let mut tasks = self.tasks.borrow_mut();
        let ts = tasks.entry(phase).or_insert(vec![]);
        ts.push(Task::new(f));
    }

    /// 毎フレーム呼び出すべき関数。
    /// 各Phaseのタスクを順に実行していく。
    pub fn update(&mut self) -> RuntimeIsDone {
        // ActivateされているPhaseをソートする
        let activated_phase = self.activated_phase.borrow();
        let mut phases = activated_phase.iter().collect::<Vec<_>>();
        phases.sort_by_key(|(&order, _phase)| order);
        let phases = phases.into_iter().map(|(_order, phase)| phase);

        for phase in phases {
            let mut tasks = self.tasks.borrow_mut();
            let tasks = tasks.entry(phase.clone()).or_insert(vec![]);

            // tasksを二分割する
            let tasks1 = tasks.split_off(tasks.len() / 2);
            let tasks2 = tasks.drain(0..).collect();

            // 分割したtasksを各スレッドに送る
            self.senders[0].send(tasks1).unwrap();
            self.senders[1].send(tasks2).unwrap();

            // スレッドからの応答を待つ
            let wait_tasks1 = self.receivers[0].recv().unwrap();
            let wait_tasks2 = self.receivers[1].recv().unwrap();
            let wait_tasks = wait_tasks1
                .into_iter()
                .chain(wait_tasks2.into_iter())
                .collect::<Vec<_>>();

            let mut wts = self.wait_tasks.borrow_mut();
            wts.insert(phase.clone(), wait_tasks);
        }

        {
            // すべてのPhaseのwait_tasksが空の場合、全てのタスクの実行が終わっている。
            let mut done_flag = true;
            let wait_tasks = self.wait_tasks.borrow();
            for (_p, tasks) in wait_tasks.iter() {
                if !tasks.is_empty() {
                    done_flag = false;
                }
            }
            if done_flag {
                return RuntimeIsDone::Done;
            }
        }

        // 次のフレームに移る前にフレームカウンターを更新する
        self.frame_counter += 1;

        // wait_tasksを空のtasks_queueと交換する
        std::mem::swap(&mut self.wait_tasks, &mut self.tasks);

        RuntimeIsDone::NotDone
    }

    /// 現在のフレームカウントを返す関数。
    /// 0スタートでカウントされている。
    pub fn frame_counter(&self) -> u64 {
        self.frame_counter
    }

    /// 実行するPhaseを登録する関数。
    /// Phaseの実行順序をorderで指定する。
    ///
    /// ActivateされたPhaseは次のフレームから実行されるようになる。
    ///
    /// ## panic
    /// orderに他のPhaseと重複した値を指定した場合、panicする。
    pub fn activate_phase(&mut self, phase: T, order: u16) {
        let mut activated_phase = self.activated_phase.borrow_mut();

        if let Some(p) = activated_phase.get(&order) {
            panic!(format!(
                "Another PHASE has already been registered in this order: {:?}",
                p
            ));
        }

        activated_phase.insert(order, phase);
    }
}
impl<T: Eq + Hash + Clone + Debug> Drop for Runtime<T> {
    fn drop(&mut self) {
        self.thread_stop_flag.store(true, Ordering::Relaxed);
        self.threads.borrow_mut()[0].take().unwrap().join().unwrap();
        self.threads.borrow_mut()[1].take().unwrap().join().unwrap();
    }
}
