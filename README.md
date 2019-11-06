# `::next_gen`

Safe generators on stable Rust.

```rust
use ::next_gen::prelude::*;

#[generator(u8)]
fn range (start: u8, end: u8)
{
    let mut current = start;
    while current < end {
        yield_!(current);
        current += 1;
    }
}

iter!(let iterator = range(3, 10));
assert_eq!(
    iterator.collect::<Vec<_>>(),
    (3 .. 10).collect::<Vec<_>>(),
);
```

## Idea

Since generators and coroutines rely on the same internals, one can derive a
safe implementation of generators using the `async` / `await` machinery, which
is only already in stable Rust.

# Credits

This crate is a fork of [whatisaphone/genawaiter](
https://github.com/whatisaphone/genawaiter), so the credits for the idea go to
[@whatisaphone](https://github.com/whatisaphone), _c.f._, [their MIT license](
https://github.com/danielhenrymantilla/next-gen-rs/blob/master/LICENSE)

Nevertheless, I have made some improvements over the implementation:

  - using `Cell` instead of `RefCell`,

  - **avoiding heap-allocations altogether**,

  - and more importantly, I've added macros and a procedural macro attribute
    to provide quite important ergonomic improvements.
