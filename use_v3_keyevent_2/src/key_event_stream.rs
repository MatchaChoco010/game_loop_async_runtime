use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;

use crossterm::event::{read, Event, KeyEvent};
use futures::Stream;

pub struct KeyEventStream {
    receiver: Receiver<KeyEvent>,
    stop_flag: Arc<AtomicBool>,
}
impl KeyEventStream {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_2 = stop_flag.clone();
        thread::spawn(move || loop {
            let evt = read();
            match evt {
                Ok(Event::Key(evt)) => sender.send(evt).unwrap(),
                Ok(_) => (),
                Err(err) => {
                    println!("{:?}", err);
                    break;
                }
            }
            if stop_flag_2.load(Ordering::Relaxed) {
                break;
            }
        });
        Self {
            receiver,
            stop_flag,
        }
    }
}
impl Drop for KeyEventStream {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}
impl Stream for KeyEventStream {
    type Item = KeyEvent;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.try_recv() {
            Ok(evt) => Poll::Ready(Some(evt)),
            Err(TryRecvError::Empty) => Poll::Pending,
            Err(TryRecvError::Disconnected) => Poll::Ready(None),
        }
    }
}
