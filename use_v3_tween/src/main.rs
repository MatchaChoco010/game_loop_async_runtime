use std::cell::RefCell;
use std::io::{stdout, Write};
use std::rc::Rc;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::Print,
    terminal::{size, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::join;

use runtime_v3::{next_frame, Runtime, RuntimeIsDone};

async fn count_up(w: Rc<RefCell<impl Write>>) {
    for i in 0..300 {
        {
            let mut w = w.borrow_mut();
            queue!(w, MoveTo(2, 5)).unwrap();
            queue!(w, Print(format!("COUNT UP: {:3}/299", i))).unwrap();
            w.flush().unwrap();
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
            w.flush().unwrap();
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
            w.flush().unwrap();
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
            w.flush().unwrap();
        }

        next_frame().await;
    }
}

fn main() {
    let mut runtime = Runtime::new();
    let stdout = Rc::new(RefCell::new(stdout()));

    runtime.spawn(async move {
        {
            let mut w = stdout.borrow_mut();
            execute!(w, EnterAlternateScreen).unwrap();
            execute!(w, Hide).unwrap();
        }

        let count_up = count_up(stdout.clone());
        let tween_1 = linear(stdout.clone());
        let tween_2 = ease_in_quadratic(stdout.clone());
        let tween_3 = ease_out_quadratic(stdout.clone());
        join!(count_up, tween_1, tween_2, tween_3);

        for _ in 0..150 {
            next_frame().await;
        }

        {
            let mut w = stdout.borrow_mut();
            execute!(w, Show).unwrap();
            execute!(w, LeaveAlternateScreen).unwrap();
        }
    });

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
