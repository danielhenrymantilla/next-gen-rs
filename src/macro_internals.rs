use_prelude!();

pub use {
    ::{
        core,
        unwind_safe,
    },
    crate::{
        generator_fn::internals::YieldSlot as __Internals_YieldSlot_DoNotUse__,
    },
};

#[cfg(feature = "alloc")]
pub extern crate alloc;

#[cfg(feature = "std")]
pub extern crate std;

macros::export_hidden_macros! {
    /* … */
}
