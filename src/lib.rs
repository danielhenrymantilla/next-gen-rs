#![cfg_attr(feature = "better-docs",
    cfg_attr(all(), doc = include_str!("lib.md")),
)]
#![cfg_attr(feature = "nightly",
    feature(doc_notable_trait),
)]
#![cfg_attr(not(feature = "better-docs"),
    doc = "See [crates.io](https://crates.io/crates/next-gen)"
)]
#![cfg_attr(not(feature = "better-docs"),
    doc = "for more info about this crate."
)]

#![allow(nonstandard_style)]
#![warn(
    missing_docs,
)]
#![deny(
    unused_must_use,
)]
#![doc(test(attr(deny(warnings), allow(unused), deny(unused_must_use))))]
#![cfg_attr(feature = "allow-warnings",
    allow(warnings),
)]

#![no_std]

#[cfg(test)]
extern crate self as next_gen;

macro_rules! use_prelude {() => (
    #[allow(unused_imports)]
    use crate::utils::prelude_internal::*;
)}

use_prelude!();

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

/// Transforms a function with `yield_!` calls into a generator.
#[cfg_attr(feature = "better-docs", cfg_attr(all(), doc = concat!(
    "\n", "# Example",
    "\n", "",
    "\n", "```rust",
    // "\n", "# const _: &str = ::core::stringify! {", "\n",
    "\n", include_str!("doc_examples/generator.rs"),
    // "\n", "# };",
    "\n", "```",
)))]
///
/// # Expansion
///
/// The above example expands to:
#[cfg_attr(feature = "better-docs", cfg_attr(all(), doc = concat!(
    "\n", "",
    "\n", "```rust",
    // "\n", "# const _: &str = ::core::stringify! {",
    "\n", include_str!("doc_examples/generator_desugared.rs"),
    // "\n", "# };",
    "\n", "```",
)))]
pub use ::next_gen_proc_macros::generator;

pub use {
    // ::{
    //     next_gen_proc_macros::generator,
    // },
    // self::{
    //     // coroutine::*,
    //     // generator::*,
    //     // ops::{Generator, GeneratorState},
    // },
};

pub mod generator;
pub mod generator_fn;
pub mod prelude;

mod iter;
mod public_macros;
mod utils;
mod waker;

#[path = "macro_internals.rs"]
#[doc(hidden)] /** Not part of the public API */ pub
mod __;

#[cfg(test)]
mod tests;
