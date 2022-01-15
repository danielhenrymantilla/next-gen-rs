//! The crate prelude: reexport the most essential utilities so that blob
//! `use`-ing them should enable the most straight-forward usage.

pub use {
    ::{
        core::{
            pin::Pin,
        },
        next_gen_proc_macros::{
            generator,
        },
    },
    crate::{
        gen_iter,
        generator::{
            Generator,
            GeneratorExt as _,
            GeneratorState,
        },
        mk_gen,
        stack_pinned,
    },
};

#[cfg(feature = "alloc")]
pub use {
    crate::{
        generator_fn::{
            CallBoxed as _,
        },
    },
};
