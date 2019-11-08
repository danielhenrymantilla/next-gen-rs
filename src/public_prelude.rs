//! The crate prelude: reexport the most essential utilities so that blob
//! `use`-ing them should enable the most straight-forward usage.

pub use crate::{
    generator,
    gen_iter,
    GeneratorState,
    mk_gen,
    stack_pinned,
};
