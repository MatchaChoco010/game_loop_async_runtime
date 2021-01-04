mod runtime;
mod wait_next_frame_future;

pub use runtime::{Runtime, RuntimeIsDone};
pub use wait_next_frame_future::next_frame;

#[cfg(test)]
mod tests {
    use super::*;
    use futures::join;

    #[test]
    fn runtime_ten_frame_single_task() {
        let mut runtime = Runtime::new();
        runtime.spawn(async {
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
        runtime.spawn(async {
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
        runtime.spawn(async {
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
}
