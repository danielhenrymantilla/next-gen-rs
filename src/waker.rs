use ::std::{
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

unsafe
fn clone (_: *const ())
  -> RawWaker
{
    RAW_WAKER
}

unsafe
fn wake (_: *const ())
{}

unsafe
fn wake_by_ref (_: *const ())
{}

unsafe
fn drop (_: *const ())
{}
