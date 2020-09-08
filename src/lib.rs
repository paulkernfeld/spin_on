#![no_std]

use core::future::Future;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use core::sync::atomic::spin_loop_hint;

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
            spin_loop_hint();
            return output;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ready() {
        crate::spin_on(async {});
    }
}
