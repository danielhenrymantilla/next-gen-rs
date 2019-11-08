# `::next_gen`

Safe generators on stable Rust.

## Examples

### Reimplementing a `range` iterator

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

mk_gen!(let generator = range(3, 10));
assert_eq!(
    generator.into_iter().collect::<Vec<_>>(),
    (3 .. 10).collect::<Vec<_>>(),
);
```

### Implementing an iterator over prime numbers using the sieve of Eratosthenes

```rust
use ::next_gen::prelude::*;

enum Void {}
type None = Option<Void>;

/// Generator over all the primes less or equal to `up_to`.
#[generator(usize)]
fn primes_up_to (up_to: usize) -> None
{
    if up_to < 2 { return None; }
    let mut sieve = vec![true; up_to.checked_add(1).expect("Overflow")];
    let mut p: usize = 1;
    loop {
        p += 1 + sieve
                    .get(p + 1 ..)?
                    .iter()
                    .position(|&is_prime| is_prime)?
        ;
        yield_!(p);
        let p2 = if let Some(p2) = p.checked_mul(p) { p2 } else {
            continue
        };
        if p2 >= up_to { continue; }
        sieve[p2 ..]
            .iter_mut()
            .step_by(p)
            .for_each(|is_prime| *is_prime = false)
        ;
    }
}

mk_gen!(let primes = primes_up_to(10_000));
for prime in primes {
    assert!(
        (2_usize ..)
            .take_while(|&n| n.saturating_mul(n) <= prime)
            .all(|n| prime % n != 0)
    );
}
```

## Features

### Performance

The crate enables no-allocation generators, thanks the usage of stack pinning.
When used in that fashion, it should thus be close to zero-cost.

### Ergonomics / sugar

A lot of effort has been put into macros and an attribute macro providing the
most ergonomic experience when dealing with these generators, despite the
complex / subtle internals involved, such as stack pinning.

### Safe

Almost no `unsafe` is used, the exception being:

  - Stack pinning, where it uses the official `::pin_utils::pin_mut`
    implementation;

  - Using the pinning guarantee to extend a lifetime;

  - A manual implementation of `Cell<Option<T>>` with a very straight-forward
    safety invariant.

### `no_std` support

This crates supports `#![no_std]`. For it, just disable the default `"alloc"`
feature:

```toml
[dependencies]
next-gen = { version = "...", default-features = false }
```

#### Idea

Since generators and coroutines rely on the same internals, one can derive a
safe implementation of generators using the `async` / `await` machinery, which
is only already in stable Rust
(credits for the idea go to [@whatisaphone](https://github.com/whatisaphone)'s
[`::genawaiter`](https://github.com/whatisaphone/genawaiter) crate, MIT licensed).
