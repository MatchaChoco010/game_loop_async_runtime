use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use futures::task::ArcWake;

use crate::container::{Container, Read};
use crate::world::World;

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
pub struct Runtime<T: Eq + Hash + Clone + Debug, W: World> {
    frame_counter: u64,
    world: Arc<Container<W>>,
    world_command_receiver: Arc<Mutex<Receiver<W::Command>>>,
    world_command_sender: Sender<W::Command>,
    tasks: Arc<Mutex<HashMap<T, Vec<Task>>>>,
    wait_tasks: Arc<Mutex<HashMap<T, Vec<Task>>>>,
    activated_phase: Arc<Mutex<HashMap<u16, T>>>,
    threads: Arc<Mutex<Vec<Option<JoinHandle<()>>>>>,
    receivers: Arc<Mutex<[Receiver<Vec<Task>>; 2]>>,
    senders: [Sender<Vec<Task>>; 2],
    thread_stop_flag: Arc<AtomicBool>,
}
impl<T: Eq + Hash + Clone + Debug, W: World> Runtime<T, W> {
    /// 新しくRuntimeを作成して返す。
    pub fn new(world: W) -> Self {
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

        let world = Arc::new(Container::new(world));
        let (world_command_sender, world_command_receiver) = channel();
        let world_command_receiver = Arc::new(Mutex::new(world_command_receiver));

        Self {
            frame_counter: 0,
            world,
            world_command_receiver,
            world_command_sender,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            wait_tasks: Arc::new(Mutex::new(HashMap::new())),
            activated_phase: Arc::new(Mutex::new(HashMap::new())),
            threads: Arc::new(Mutex::new(vec![Some(thread1), Some(thread2)])),
            receivers: Arc::new(Mutex::new([main_receiver1, main_receiver2])),
            senders: [main_sender1, main_sender2],
            thread_stop_flag,
        }
    }

    /// 非同期のタスクを登録する関数。
    /// 非同期関数は[`Read<World>`]と[`Sender<World::Command>`]、[`Runtime`]を受け取る。
    pub fn add_async_system<F, Fut>(&self, phase: T, f: F)
    where
        F: FnOnce(Read<W>, Sender<W::Command>, Self) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut wait_tasks = self.wait_tasks.lock().unwrap();
        let wts = wait_tasks.entry(phase).or_insert(vec![]);
        wts.push(Task::new(f(
            unsafe { self.world.read() },
            self.world_command_sender.clone(),
            self.clone(),
        )));
    }

    /// 毎フレーム呼び出すべき関数。
    /// 各Phaseのタスクを順に実行していく。
    pub fn update(&mut self) -> RuntimeIsDone {
        // ActivateされているPhaseをソートする
        let activated_phase = self.activated_phase.lock().unwrap();
        let mut phases = activated_phase.iter().collect::<Vec<_>>();
        phases.sort_by_key(|(&order, _phase)| order);
        let phases = phases.into_iter().map(|(_order, phase)| phase);

        // wait_tasksを空のtasks_queueと交換する
        std::mem::swap(&mut self.wait_tasks, &mut self.tasks);

        // Phaseにについてループする
        for phase in phases {
            let (tasks1, tasks2) = {
                let mut tasks = self.tasks.lock().unwrap();
                let tasks = tasks.entry(phase.clone()).or_insert(vec![]);

                // tasksを二分割する
                let tasks1 = tasks.split_off(tasks.len() / 2);
                let tasks2 = tasks.drain(0..).collect();
                (tasks1, tasks2)
            };

            // 分割したtasksを各スレッドに送る
            self.senders[0].send(tasks1).unwrap();
            self.senders[1].send(tasks2).unwrap();

            // スレッドからの応答を待つ
            let receivers = self.receivers.lock().unwrap();
            let wait_tasks1 = receivers[0].recv().unwrap();
            let wait_tasks2 = receivers[1].recv().unwrap();

            // 各スレッドから帰ってきたwait_tasksを追加する。
            let wait_tasks = wait_tasks1.into_iter().chain(wait_tasks2.into_iter());
            let mut wts = self.wait_tasks.lock().unwrap();
            let wts_entry = wts.entry(phase.clone()).or_insert(vec![]);
            for t in wait_tasks {
                wts_entry.push(t);
            }

            // このphaseで送信されたコマンドを直列で実行する
            {
                let mut world = unsafe { self.world.write() };
                for cmd in self.world_command_receiver.lock().unwrap().try_iter() {
                    world.process_command(cmd);
                }
            }
        }

        {
            // すべてのPhaseのwait_tasksが空の場合、全てのタスクの実行が終わっている。
            let mut done_flag = true;
            let wait_tasks = self.wait_tasks.lock().unwrap();
            for (_p, tasks) in wait_tasks.iter() {
                if !tasks.is_empty() {
                    done_flag = false;
                }
            }

            if done_flag {
                self.thread_stop_flag.store(true, Ordering::Relaxed);
                let mut threads = self.threads.lock().unwrap();
                threads[0].take().unwrap().join().unwrap();
                threads[1].take().unwrap().join().unwrap();

                return RuntimeIsDone::Done;
            }
        }

        // 次のフレームに移る前にフレームカウンターを更新する
        self.frame_counter += 1;

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
        let mut activated_phase = self.activated_phase.lock().unwrap();

        if let Some(p) = activated_phase.get(&order) {
            panic!(format!(
                "Another PHASE has already been registered in this order: {:?}",
                p
            ));
        }

        activated_phase.insert(order, phase);
    }
}
// deriveマクロではWorldに過剰なCloneが要求されてしまうので手動で実装する。
// https://qnighy.hatenablog.com/entry/2017/06/01/070000
impl<T: Eq + Hash + Clone + Debug, W: World> Clone for Runtime<T, W> {
    fn clone(&self) -> Self {
        Self {
            frame_counter: self.frame_counter,
            world: Arc::clone(&self.world),
            world_command_receiver: Arc::clone(&self.world_command_receiver),
            world_command_sender: self.world_command_sender.clone(),
            tasks: Arc::clone(&self.tasks),
            wait_tasks: Arc::clone(&self.wait_tasks),
            activated_phase: Arc::clone(&self.activated_phase),
            threads: Arc::clone(&self.threads),
            receivers: Arc::clone(&self.receivers),
            senders: self.senders.clone(),
            thread_stop_flag: Arc::clone(&self.thread_stop_flag),
        }
    }
}
