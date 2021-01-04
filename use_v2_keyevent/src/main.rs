use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crossterm::{
    event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;

use runtime_v2::{Runtime, RuntimeIsDone};

// runtime_v2はawaitすると問答無用で次のフレームに処理が回ってしまう。
// そのためawaitでキーイベントのストリームを待つと1フレームに1つのキーしか処理できない。
async fn print_key_event() {
    let mut reader = EventStream::new();

    while let Some(evt) = reader.next().await {
        match evt {
            Ok(event) => match event {
                Event::Key(key_event) => {
                    println!("{:?}", key_event);
                    match key_event {
                        KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers: KeyModifiers::CONTROL,
                        } => break,
                        _ => (),
                    }
                }
                _ => (),
            },
            Err(err) => {
                println!("{:?}", err);
            }
        }
    }
}

fn main() {
    let mut runtime = Runtime::new();

    enable_raw_mode().unwrap();

    runtime.spawn(async {
        print_key_event().await;
    });

    'update_loop: loop {
        let frame_start = Instant::now();
        let frame_duration = Duration::new(1, 0);

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

    disable_raw_mode().unwrap();
}
