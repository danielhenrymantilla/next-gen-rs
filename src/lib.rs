#![cfg_attr(feature = "external_doc",
    feature(external_doc)
)]
#![cfg_attr(feature = "external_doc",
    doc(include = "../README.md")
)]
#![cfg_attr(not(feature = "external_doc"),
    doc = "See [crates.io](https://crates.io/crates/next-gen)"
)]
#![cfg_attr(not(feature = "external_doc"),
    doc = "for more info about this crate."
)]

#![warn(
    future_incompatible,
    rust_2018_compatibility,
    missing_docs,
    clippy::cargo,
    clippy::pedantic,
)]
#![deny(
    unused_must_use,
)]
#![doc(test(attr(deny(warnings))))]
#![cfg_attr(feature = "allow-warnings",
    allow(warnings),
)]

#![cfg_attr(not(feature = "std"),
    no_std,
)]

#[cfg(feature = "std")]
pub extern crate alloc;

#[path = "public_prelude.rs"]
pub
mod prelude;

mod public_macros;

#[macro_use]
mod utils;

mod iter;

mod waker;

pub use self::generator::*;
mod generator;

#[doc(hidden)] pub use ::core;
#[doc(hidden)] pub use ::proc_macro::next_gen_hack;
pub use ::proc_macro::generator;

#[cfg(test)]
mod tests;
