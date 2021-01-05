use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;

use runtime_v3::{Runtime, RuntimeIsDone};

mod key_event_stream;
use key_event_stream::KeyEventStream;

// 改めてKeyEventStreamを自作して実装する。
//
// runtime_v2ではawaitするたびに問答無用で後続タスクが次フレームに送られるので
// 1フレームに1つのキーしか処理することができない。
// runtime_v3はawaitしても、皇族タスクが即座に実行可能な場合には次フレームに送らず
// 同フレーム中で後続タスクを続けて処理をする。
// 今回はruntime_v3を使っているので、後続処理が即座に実行可能な場合、
// つまりキーイベントストリームにキーイベントが溜まっていた場合には、
// 同フレーム中で処理を回せるため、1フレームで複数のキーイベントを処理できる。
async fn print_key_event() {
    let mut stream = KeyEventStream::new();
    while let Some(evt) = stream.next().await {
        println!("{:?}", evt);
        match evt {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            } => break,
            _ => (),
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
