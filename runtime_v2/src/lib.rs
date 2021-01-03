mod runtime;
mod wait_next_frame_future;

pub use runtime::{Runtime, RuntimeIsDone};
pub use wait_next_frame_future::next_frame;

#[cfg(test)]
mod tests {
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    use super::*;
    use futures::join;

    #[test]
    fn runtime_ten_frame_single_task() {
        let mut runtime = Runtime::new();
        runtime.spawn(async {
            for i in 0..9 {
                println!("Task 1: frame {}", i);
                next_frame().await;
            }
        });

        assert_eq!(runtime.frame_counter() + 1, 1);

        'update_loop: loop {
            let frame_start = Instant::now();
            let frame_duration = Duration::new(0, 16_666_666);

            let done = runtime.update();

            match done {
                RuntimeIsDone::Done => break 'update_loop,
                RuntimeIsDone::NotDone => (),
            }

            let now = Instant::now();
            let duration = now.duration_since(frame_start);
            if duration < frame_duration {
                sleep(frame_duration - duration);
            }
        }

        assert_eq!(runtime.frame_counter() + 1, 10);
    }

    #[test]
    fn runtime_ten_frame_concurrent_multi_task() {
        async fn ten_frame_task(task_id: u8) {
            for i in 0..9 {
                println!("Task {}: frame {}", task_id, i);
                next_frame().await;
            }
        }

        let mut runtime = Runtime::new();
        runtime.spawn(async {
            join!(ten_frame_task(0), ten_frame_task(1));
        });

        assert_eq!(runtime.frame_counter() + 1, 1);

        'update_loop: loop {
            let frame_start = Instant::now();
            let frame_duration = Duration::new(0, 16_666_666);

            let done = runtime.update();

            match done {
                RuntimeIsDone::Done => break 'update_loop,
                RuntimeIsDone::NotDone => (),
            }

            let now = Instant::now();
            let duration = now.duration_since(frame_start);
            if duration < frame_duration {
                sleep(frame_duration - duration);
            }
        }

        assert_eq!(runtime.frame_counter() + 1, 10);
    }
}
