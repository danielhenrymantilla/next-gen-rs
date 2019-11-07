# `::next_gen`

Safe generators on stable Rust.

## Examples

Reimplementing a `range` iterator:

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

mk_gen!(let mut generator = range(3, 10));
assert_eq!(
    generator.into_iter().collect::<Vec<_>>(),
    (3 .. 10).collect::<Vec<_>>(),
);
```

Implementing an iterator over prime numbers using the sieve of Eratosthenes

```rust
use ::next_gen::prelude::*;

enum Void {}
type None = Option<Void>;

/// Generator over all the primes less or equal to `up_to`.
#[generator(usize)]
fn primes_up_to (up_to: usize) -> None
{
    if up_to < 2 { return None; }
    let mut sieve = vec![true; up_to + 1];
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

mk_gen!(let primes = primes_up_to(11));
assert_eq!(
    primes.into_iter().collect::<Vec<_>>(),
    [2, 3, 5, 7, 11],
);
```

#### Idea

Since generators and coroutines rely on the same internals, one can derive a
safe implementation of generators using the `async` / `await` machinery, which
is only already in stable Rust
(credits for the idea go to [@whatisaphone](https://github.com/whatisaphone)'s
[`::genawaiter`](https://github.com/whatisaphone/genawaiter) crate, MIT licensed).
