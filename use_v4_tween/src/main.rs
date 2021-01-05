use std::cell::RefCell;
use std::io::{stdout, Write};
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
    terminal::{size, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::join;

use runtime_v4::{next_frame, Runtime, RuntimeIsDone};

async fn count_up(w: Rc<RefCell<impl Write>>) {
    for i in 0..300 {
        {
            let mut w = w.borrow_mut();
            queue!(w, MoveTo(2, 5)).unwrap();
            queue!(w, Print(format!("COUNT UP: {:3}/299", i))).unwrap();
        }

        next_frame().await;
    }
}

async fn linear(w: Rc<RefCell<impl Write>>) {
    for i in 0..300 {
        let t = i as f32 / 299 as f32;

        let width = size().unwrap().0;
        let bar_width = ((width - 2) as f32 * t) as u16;

        let mut progress_bar = "".to_string();
        for _ in 0..bar_width {
            progress_bar.push('#');
        }
        for _ in 0..(width - 2 - bar_width) {
            progress_bar.push(' ');
        }

        {
            let mut w = w.borrow_mut();
            queue!(w, MoveTo(2, 7), Print("Linear:")).unwrap();
            queue!(w, MoveTo(0, 8), Print(format!("[{}]", progress_bar))).unwrap();
        }

        next_frame().await;
    }
}

async fn ease_in_quadratic(w: Rc<RefCell<impl Write>>) {
    fn easing_ease_in_quadratic(t: f32) -> f32 {
        t * t
    }

    for i in 0..300 {
        let t = i as f32 / 299 as f32;
        let t = easing_ease_in_quadratic(t);

        let width = size().unwrap().0;
        let bar_width = ((width - 2) as f32 * t) as u16;

        let mut progress_bar = "".to_string();
        for _ in 0..bar_width {
            progress_bar.push('#');
        }
        for _ in 0..(width - 2 - bar_width) {
            progress_bar.push(' ');
        }

        {
            let mut w = w.borrow_mut();
            queue!(w, MoveTo(2, 10), Print("EaseInQuadratic:")).unwrap();
            queue!(w, MoveTo(0, 11), Print(format!("[{}]", progress_bar))).unwrap();
        }

        next_frame().await;
    }
}

async fn ease_out_quadratic(w: Rc<RefCell<impl Write>>) {
    fn easing_ease_out_quadratic(t: f32) -> f32 {
        -t * (t - 2.0)
    }

    for i in 0..300 {
        let t = i as f32 / 299 as f32;
        let t = easing_ease_out_quadratic(t);

        let width = size().unwrap().0;
        let bar_width = ((width - 2) as f32 * t) as u16;

        let mut progress_bar = "".to_string();
        for _ in 0..bar_width {
            progress_bar.push('#');
        }
        for _ in 0..(width - 2 - bar_width) {
            progress_bar.push(' ');
        }

        {
            let mut w = w.borrow_mut();
            queue!(w, MoveTo(2, 13), Print("EaseOutQuadratic:")).unwrap();
            queue!(w, MoveTo(0, 14), Print(format!("[{}]", progress_bar))).unwrap();
        }

        next_frame().await;
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum Phase {
    QueueCommand,
    Flush,
}

async fn queue_command(w: Rc<RefCell<impl Write>>) {
    {
        let mut w = w.borrow_mut();
        execute!(w, EnterAlternateScreen).unwrap();
        execute!(w, Hide).unwrap();
    }

    let count_up = count_up(w.clone());
    let tween_1 = linear(w.clone());
    let tween_2 = ease_in_quadratic(w.clone());
    let tween_3 = ease_out_quadratic(w.clone());
    join!(count_up, tween_1, tween_2, tween_3);

    for _ in 0..150 {
        next_frame().await;
    }

    {
        let mut w = w.borrow_mut();
        execute!(w, Show).unwrap();
        execute!(w, LeaveAlternateScreen).unwrap();
    }
}

async fn flush(w: Rc<RefCell<impl Write>>) {
    loop {
        {
            let mut w = w.borrow_mut();
            w.flush().unwrap();
        }
        next_frame().await;
    }
}

fn main() {
    let mut runtime = Runtime::new();
    let stdout = Rc::new(RefCell::new(stdout()));

    runtime.activate_phase(Phase::QueueCommand, 0);
    runtime.activate_phase(Phase::Flush, 10);

    runtime.spawn(Phase::QueueCommand, queue_command(stdout.clone()));
    runtime.spawn(Phase::Flush, flush(stdout.clone()));

    'update_loop: loop {
        let frame_start = Instant::now();
        let frame_duration = Duration::new(0, 16_666_666);

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
