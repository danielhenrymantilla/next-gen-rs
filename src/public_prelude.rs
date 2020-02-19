//! The crate prelude: reexport the most essential utilities so that blob
//! `use`-ing them should enable the most straight-forward usage.

pub use crate::{
    generator,
    gen_iter,
    Generator,
    GeneratorState,
    mk_gen,
    stack_pinned,
};
pub use ::core::pin::Pin;
