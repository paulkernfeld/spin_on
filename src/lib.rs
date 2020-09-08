//! This crate contains what aims to be the simplest possible implementation of a valid executor.
//! Instead of nicely parking the thread and waiting for the future to wake it up, it continuously
//! polls the future until the future is ready. This will probably use a lot of CPU, so be careful
//! when you use it.
//!
//! The advantages of this crate are:
//!
//! - It is really simple
//! - It has no dependency on `std` or on an allocator
//! - It only has one dependency
#![no_std]

use core::future::Future;
use core::sync::atomic::spin_loop_hint;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

// TODO audit that this noop waker implementations aren't doing anything bad
unsafe fn rwclone(_p: *const ()) -> RawWaker {
    noop_waker()
}

unsafe fn rwwake(_p: *const ()) {}

unsafe fn rwwakebyref(_p: *const ()) {}

unsafe fn rwdrop(_p: *const ()) {}

static VTABLE: RawWakerVTable = RawWakerVTable::new(rwclone, rwwake, rwwakebyref, rwdrop);

/// The simplest way to create a noop waker in Rust. You would only ever want to use this with
/// an executor that polls continuously. Thanks to user 2e71828 on
/// [this Rust forum post](https://users.rust-lang.org/t/simplest-possible-block-on/48364/2).
fn noop_waker() -> RawWaker {
    static DATA: () = ();
    RawWaker::new(&DATA, &VTABLE)
}

/// Continuously poll a future until it returns `Poll::Ready`. This is not normally how an
/// executor should work, because it runs the CPU at 100%.
pub fn spin_on<F: Future>(future: F) -> F::Output {
    pin_utils::pin_mut!(future);
    let waker = &unsafe { Waker::from_raw(noop_waker()) };
    let mut cx = Context::from_waker(waker);
    loop {
        if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
            spin_loop_hint(); // TODO am I using this correctly?
            return output;
        }
    }
}

#[cfg(test)]
mod tests {
    use core::future::Future;
    use core::pin::Pin;
    use core::task::{Context, Poll};

    struct CountFuture(usize);

    impl Future for CountFuture {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.0 > 0 {
                self.0 -= 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        }
    }

    #[test]
    fn ready() {
        crate::spin_on(async {});
    }

    #[test]
    fn count() {
        crate::spin_on(CountFuture(10));
    }
}
