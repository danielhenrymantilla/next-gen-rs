use ::core::{
    task::{RawWaker, RawWakerVTable, Waker},
};

pub
fn create ()
  -> Waker
{
    unsafe {
        // # Safety
        //
        //   - Waker has no context so all the provided methods are no-ops
        //     and thus sound.
        Waker::from_raw(RAW_WAKER)
    }
}

const RAW_WAKER: RawWaker =
    RawWaker::new(0 as _, &VTABLE)
;

const VTABLE: RawWakerVTable =
    RawWakerVTable::new(clone, wake, wake_by_ref, drop)
;

unsafe // Safety: no-op function
fn clone (_: *const ())
  -> RawWaker
{
    RAW_WAKER
}

unsafe // Safety: no-op function
fn wake (_: *const ())
{}

unsafe // Safety: no-op function
fn wake_by_ref (_: *const ())
{}

unsafe // Safety: no-op function
fn drop (_: *const ())
{}
