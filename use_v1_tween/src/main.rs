use std::cell::RefCell;
use std::io::{stdout, Write};
use std::rc::Rc;

use futures::join;
use termion::{
    cursor::{Goto, HideCursor},
    screen::AlternateScreen,
    terminal_size,
};

use runtime_v1::{next_frame, Runtime};

async fn count_up(screen: Rc<RefCell<impl Write>>) {
    for i in 0..300 {
        {
            let mut screen = screen.borrow_mut();
            write!(screen, "{}COUNT UP: {:3}/299", Goto(2, 5), i).unwrap();
            screen.flush().unwrap();
        }

        next_frame().await;
    }
}

async fn linear(screen: Rc<RefCell<impl Write>>) {
    for i in 0..300 {
        let t = i as f32 / 299 as f32;

        let width = terminal_size().unwrap().0;
        let bar_width = ((width - 3) as f32 * t) as u16;

        let mut progress_bar = "".to_string();
        for _ in 0..bar_width {
            progress_bar.push('#');
        }
        for _ in 0..(width - 3 - bar_width) {
            progress_bar.push(' ');
        }

        {
            let mut screen = screen.borrow_mut();
            write!(screen, "{}Linear:", Goto(2, 7)).unwrap();
            write!(screen, "{}[{}] ", Goto(2, 8), progress_bar).unwrap();
            screen.flush().unwrap();
        }

        next_frame().await;
    }
}

fn easing_ease_in_cubic(t: f32) -> f32 {
    t * t * t
}

async fn ease_in_cubic(screen: Rc<RefCell<impl Write>>) {
    for i in 0..300 {
        let t = i as f32 / 299 as f32;
        let t = easing_ease_in_cubic(t);

        let width = terminal_size().unwrap().0;
        let bar_width = ((width - 3) as f32 * t) as u16;

        let mut progress_bar = "".to_string();
        for _ in 0..bar_width {
            progress_bar.push('#');
        }
        for _ in 0..(width - 3 - bar_width) {
            progress_bar.push(' ');
        }

        {
            let mut screen = screen.borrow_mut();
            write!(screen, "{}EaseInCubic:", Goto(2, 10)).unwrap();
            write!(screen, "{}[{}] ", Goto(2, 11), progress_bar).unwrap();
            screen.flush().unwrap();
        }

        next_frame().await;
    }
}

fn easing_ease_out_cubic(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}
async fn ease_out_cubic(screen: Rc<RefCell<impl Write>>) {
    for i in 0..300 {
        let t = i as f32 / 299 as f32;
        let t = easing_ease_out_cubic(t);

        let width = terminal_size().unwrap().0;
        let bar_width = ((width - 3) as f32 * t) as u16;

        let mut progress_bar = "".to_string();
        for _ in 0..bar_width {
            progress_bar.push('#');
        }
        for _ in 0..(width - 3 - bar_width) {
            progress_bar.push(' ');
        }

        {
            let mut screen = screen.borrow_mut();
            write!(screen, "{}EaseOutCubic:", Goto(2, 13)).unwrap();
            write!(screen, "{}[{}] ", Goto(2, 14), progress_bar).unwrap();
            screen.flush().unwrap();
        }

        next_frame().await;
    }
}

fn main() {
    let mut runtime = Runtime::new();
    runtime.spawn(async {
        let screen = HideCursor::from(AlternateScreen::from(stdout()));
        let screen = Rc::new(RefCell::new(screen));

        let tween1 = count_up(screen.clone());
        let tween2 = linear(screen.clone());
        let tween3 = ease_in_cubic(screen.clone());
        let tween4 = ease_out_cubic(screen.clone());
        join!(tween1, tween2, tween3, tween4);

        let tween2 = linear(screen.clone());
        tween2.await;

        let tween3 = ease_in_cubic(screen.clone());
        let tween4 = ease_out_cubic(screen);
        join!(tween3, tween4);

        for _ in 0..120 {
            next_frame().await;
        }
    });
    runtime.run();
}
