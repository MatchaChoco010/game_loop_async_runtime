mod runtime;
mod wait_next_frame_future;

pub use runtime::{Runtime, RuntimeIsDone};
pub use wait_next_frame_future::next_frame;

#[cfg(test)]
mod tests {
    use super::*;
    use futures::join;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[derive(Eq, PartialEq, Clone, Hash, Debug)]
    enum Phase {
        Phase1,
        Phase2,
    }

    #[test]
    fn runtime_ten_frame_single_task() {
        let mut runtime = Runtime::new();
        runtime.activate_phase(Phase::Phase1, 0);
        runtime.spawn(Phase::Phase1, async {
            for i in 0..10 {
                println!("Task 1: frame {}", i);
                next_frame().await;
            }
        });

        assert_eq!(runtime.frame_counter(), 0);

        'update_loop: loop {
            match runtime.update() {
                RuntimeIsDone::Done => break 'update_loop,
                RuntimeIsDone::NotDone => (),
            }
        }

        assert_eq!(runtime.frame_counter(), 10);
    }

    #[test]
    fn runtime_ten_frame_concurrent_multi_task() {
        async fn ten_frame_task(task_id: u8) {
            for i in 0..10 {
                println!("Task {}: frame {}", task_id, i);
                next_frame().await;
            }
        }

        let mut runtime = Runtime::new();
        runtime.activate_phase(Phase::Phase1, 0);
        runtime.spawn(Phase::Phase1, async {
            join!(ten_frame_task(0), ten_frame_task(1));
        });

        assert_eq!(runtime.frame_counter(), 0);

        'update_loop: loop {
            match runtime.update() {
                RuntimeIsDone::Done => break 'update_loop,
                RuntimeIsDone::NotDone => (),
            }
        }

        assert_eq!(runtime.frame_counter(), 10);
    }

    #[test]
    fn runtime_await_0_frame_task_should_cost_0_frame() {
        let mut runtime = Runtime::new();
        runtime.activate_phase(Phase::Phase1, 0);
        runtime.spawn(Phase::Phase1, async {
            let x = async { 21 }.await;
            let y = async { 21 }.await;
            println!("{}", x + y);
        });

        assert_eq!(runtime.frame_counter(), 0);

        'update_loop: loop {
            match runtime.update() {
                RuntimeIsDone::Done => break 'update_loop,
                RuntimeIsDone::NotDone => (),
            }
        }

        assert_eq!(runtime.frame_counter(), 0);
    }

    #[test]
    fn not_activated_task_should_not_call() {
        let mut runtime = Runtime::new();
        runtime.activate_phase(Phase::Phase1, 0);

        runtime.spawn(Phase::Phase2, async {
            panic!("should not call this phase!");
        });

        'update_loop: loop {
            match runtime.update() {
                RuntimeIsDone::Done => break 'update_loop,
                RuntimeIsDone::NotDone => (),
            }
        }
    }

    #[test]
    fn call_task_in_the_order_of_phase() {
        let mut runtime = Runtime::new();
        runtime.activate_phase(Phase::Phase1, 0);
        runtime.activate_phase(Phase::Phase2, 1);

        let flag = Arc::new(AtomicBool::new(false));

        let flag2 = Arc::clone(&flag);
        runtime.spawn(Phase::Phase2, async move {
            // this should call after phase 1
            let flag = flag2.load(Ordering::Relaxed);
            assert!(flag);
        });

        let flag1 = Arc::clone(&flag);
        runtime.spawn(Phase::Phase1, async move {
            flag1.store(true, Ordering::Relaxed);
        });

        'update_loop: loop {
            match runtime.update() {
                RuntimeIsDone::Done => break 'update_loop,
                RuntimeIsDone::NotDone => (),
            }
        }
    }

    #[test]
    #[should_panic(expected = "Another PHASE has already been registered in this order: Phase1")]
    fn phase_order_num_should_different_from_other_phases() {
        let mut runtime = Runtime::new();
        runtime.activate_phase(Phase::Phase1, 0);

        // this line should panic
        runtime.activate_phase(Phase::Phase2, 0);
    }
}
