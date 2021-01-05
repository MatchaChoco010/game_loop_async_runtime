use std::pin::Pin;
use std::task::Context;
use std::{future::Future, task::Poll};

pub struct WaitNextFrameFuture {
    polled: bool,
}
impl WaitNextFrameFuture {
    fn new() -> Self {
        Self { polled: false }
    }
}
impl Future for WaitNextFrameFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled {
            Poll::Ready(())
        } else {
            self.polled = true;
            Poll::Pending
        }
    }
}

/// 次のフレームまで待機するFutureを返す関数。
pub fn next_frame() -> WaitNextFrameFuture {
    WaitNextFrameFuture::new()
}
