# `::next_gen`

Safe generators on stable Rust.

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/next-gen.rs)
[![Latest version](https://img.shields.io/crates/v/next-gen.svg)](
https://crates.io/crates/next-gen)
[![Documentation](https://docs.rs/next-gen/badge.svg)](
https://docs.rs/next-gen)
[![MSRV](https://img.shields.io/badge/MSRV-1.45.0-white)](
https://gist.github.com/danielhenrymantilla/8e5b721b3929084562f8f65668920c33)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/next-gen.svg)](
https://github.com/danielhenrymantilla/next-gen.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/next-gen.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/next-gen.rs/actions)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

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


### Defining an iterator with borrowed state

This is surely the most useful feature of a generator.

For instance, the following does not work, no matter how hard you try:

```rust,compile_fail
use ::std::sync::Mutex;

fn iter_locked (vec: &'_ Mutex<Vec<i32>>)
  -> impl Iterator<Item = i32> + '_
{
    ::std::iter::from_fn({
        let guard = vec.lock().unwrap();
        let mut iter = guard.iter().copied();
        move || {
            // let _ = guard;
            iter.next()
        }
    })
}
```

But this works:

```rust
use ::next_gen::prelude::*;
use ::std::sync::Mutex;

fn iter_locked (vec: &'_ Mutex<Vec<i32>>) -> impl Iterator<Item = i32> + '_
{
    #[generator(i32)]
    fn gen (mutex: &'_ Mutex<Vec<i32>>)
    {
        let vec = mutex.lock().unwrap();
        for &elem in vec.iter() {
            yield_!(elem);
        }
    }
    mk_gen!(let generator = box gen(vec));
    generator
        .into_iter()
}

let vec = Mutex::new(vec![42, 27]);
let mut iter = iter_locked(&vec);
assert_eq!(iter.next(), Some(42));
assert_eq!(iter.next(), Some(27));
assert_eq!(iter.next(), None);
```

  - If the `iter_locked()` function you are trying to implement is part of
    a trait definition and thus need to name the type, you can use
    `Pin<Box<dyn Generator<(), Yield = i32, Return = ()> + '_>>`

    ```rust
    use ::next_gen::prelude::*;

    struct Once<T>(T);
    impl<T : 'static> IntoIterator for Once<T> {
        type Item = T;
        type IntoIter = Pin<Box<dyn Generator<(), Yield = T, Return = ()> + 'static>>;

        fn into_iter (self: Once<T>) -> Self::IntoIter
        {
            #[generator(T)]
            fn gen<T> (Once(value): Once<T>)
            {
                yield_!(value);
            }
            mk_gen!(let generator = box gen(self));
            generator
        }
    }
    assert_eq!(Once(42).into_iter().next(), Some(42));
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

This crates supports `#![no_std]`. For it, just disable the default `"std"`
feature:

```toml
[dependencies]
next-gen = { version = "...", default-features = false }
```

#### Idea

Since generators and coroutines rely on the same internals, one can derive a
safe implementation of generators using the `async` / `await` machinery, which
is only already in stable Rust.

A similar idea has also been implemented in <https://docs.rs/genawaiter>.
