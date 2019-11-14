use ::core::{
    task::{RawWaker, RawWakerVTable, Waker},
};

pub
fn create ()
  -> Waker
{
    const RAW_WAKER: RawWaker = {
        const VTABLE: RawWakerVTable = {
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

            RawWakerVTable::new(clone, wake, wake_by_ref, drop)
        };

        RawWaker::new(0 as _, &VTABLE)
    };

    unsafe {
        // # Safety
        //
        //   - Waker has no context so all the provided methods are no-ops
        //     and thus sound.
        Waker::from_raw(RAW_WAKER)
    }
}
