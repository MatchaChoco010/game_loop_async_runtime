use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyEvent};

pub struct KeyEvents {
    receiver: Receiver<KeyEvent>,
    stop_flag: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}
impl KeyEvents {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_2 = stop_flag.clone();
        let join_handle = thread::spawn(move || loop {
            if poll(Duration::from_millis(16)).unwrap() {
                let evt = read();
                match evt {
                    Ok(Event::Key(evt)) => sender.send(evt).unwrap(),
                    Ok(_) => (),
                    Err(err) => {
                        println!("{:?}", err);
                        break;
                    }
                }
            }
            if stop_flag_2.load(Ordering::Relaxed) {
                break;
            }
        });
        Self {
            receiver,
            stop_flag,
            join_handle: Some(join_handle),
        }
    }

    pub fn get_events(&mut self) -> Vec<KeyEvent> {
        let mut v = vec![];
        for evt in self.receiver.try_iter() {
            v.push(evt);
        }
        v
    }
}
impl Drop for KeyEvents {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.join_handle.take().unwrap().join().unwrap();
    }
}
