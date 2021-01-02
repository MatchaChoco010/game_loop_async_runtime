use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct WaitNextFrame {
    polled: bool,
}
impl WaitNextFrame {
    fn new() -> Self {
        Self { polled: false }
    }
}
impl Future for WaitNextFrame {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled {
            Poll::Ready(())
        } else {
            self.get_mut().polled = true;
            Poll::Pending
        }
    }
}

pub fn next_frame() -> WaitNextFrame {
    WaitNextFrame::new()
}
