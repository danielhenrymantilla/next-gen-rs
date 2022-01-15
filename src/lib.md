# `::next_gen`

Safe generators on stable Rust.

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/next-gen-rs)
[![Latest version](https://img.shields.io/crates/v/next-gen.svg)](
https://crates.io/crates/next-gen)
[![Documentation](https://docs.rs/next-gen/badge.svg)](
https://docs.rs/next-gen)
[![MSRV](https://img.shields.io/badge/MSRV-1.45.0-white)](
https://gist.github.com/danielhenrymantilla/8e5b721b3929084562f8f65668920c33)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/next-gen.svg)](
https://github.com/danielhenrymantilla/next-gen-rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/next-gen-rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/next-gen-rs/actions)

## Examples

### Reimplementing a `range` iterator

```rust
use ::next_gen::prelude::*;

#[generator(yield(u8))]
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

enum NeverSome {}

/// Generator over all the primes less or equal to `up_to`.
#[generator(yield(usize))]
fn primes_up_to (up_to: usize)
  -> Option<NeverSome>
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


### Defining an iterator with self-borrowed state

This is surely the most useful feature of a generator.

Consider, for instance, the following problem:

```rust
# #[cfg(any)] macro_rules! ignore {
fn iter_locked (elems: &'_ Mutex<Set<i32>>)
  -> impl '_ + Iterator<Item = i32>
# }
```

#### Miserable attempts without generators

No matter how hard you try, without using `unsafe`, or some other
`unsafe`-using self-referential library/tool, you won't be able to feature such
a signature!

  - The following fails:

    ```rust ,compile_fail
    # use ::std::{
    #     collections::BTreeSet as Set,
    #     sync::Mutex,
    # };
    #
    fn iter_locked (mutexed_elems: &'_ Mutex<Set<i32>>)
      -> impl '_ + Iterator<Item = i32>
    {
        ::std::iter::from_fn({
            let locked_elems = mutexed_elems.lock().unwrap();
            let mut elems = locked_elems.iter().copied();
            move || {
                // let _ = locked_elems;
                elems.next()
            } // Error, borrowed `locked_elems` is not captured and is thus dropped!
        })
    }
    ```

    <details>

    ```rust ,ignore
    error[E0515]: cannot return value referencing local variable `locked_elems`
      --> src/lib.rs:122:5
       |
    11 | /     ::std::iter::from_fn({
    12 | |         let locked_elems = mutexed_elems.lock().unwrap();
    13 | |         let mut elems = locked_elems.iter().copied();
       | |                         ------------------- `locked_elems` is borrowed here
    14 | |         move || {
    ...  |
    17 | |         } // Error, borrowed `locked_elems` is not captured and is thus dropped!
    18 | |     })
       | |______^ returns a value referencing data owned by the current function
       |
       = help: use `.collect()` to allocate the iterator
    ```

    </details>

  - as well as this:

    ```rust ,compile_fail
    # use ::std::{
    #     collections::BTreeSet as Set,
    #     sync::Mutex,
    # };
    #
    fn iter_locked (mutexed_elems: &'_ Mutex<Set<i32>>)
      -> impl '_ + Iterator<Item = i32>
    {
        ::std::iter::from_fn({
            let locked_elems = mutexed_elems.lock().unwrap();
            let mut elems = locked_elems.iter().copied();
            move || {
                let _ = &locked_elems; // ensure `locked_elems` is captured (and thus moved)
                elems.next() // Error, can't use borrow of moved value!
            }
        })
    }
    ```

    <details>

    ```rust ,ignore
    error[E0515]: cannot return value referencing local variable `locked_elems`
      --> src/lib.rs:144:5
       |
    11 | /     ::std::iter::from_fn({
    12 | |         let locked_elems = mutexed_elems.lock().unwrap();
    13 | |         let mut elems = locked_elems.iter().copied();
       | |                         ------------------- `locked_elems` is borrowed here
    14 | |         move || {
    ...  |
    17 | |         }
    18 | |     })
       | |______^ returns a value referencing data owned by the current function
       |
       = help: use `.collect()` to allocate the iterator

    error[E0505]: cannot move out of `locked_elems` because it is borrowed
      --> src/lib.rs:147:9
       |
    8  |   fn iter_locked (mutexed_elems: &'_ Mutex<Set<i32>>)
       |                                  - let's call the lifetime of this reference `'1`
    ...
    11 | /     ::std::iter::from_fn({
    12 | |         let locked_elems = mutexed_elems.lock().unwrap();
    13 | |         let mut elems = locked_elems.iter().copied();
       | |                         ------------------- borrow of `locked_elems` occurs here
    14 | |         move || {
       | |         ^^^^^^^ move out of `locked_elems` occurs here
    15 | |             let _ = &locked_elems; // ensure `locked_elems` is captured (and thus moved)
       | |                      ------------ move occurs due to use in closure
    16 | |             elems.next() // Error, can't use borrow of moved value!
    17 | |         }
    18 | |     })
       | |______- returning this value requires that `locked_elems` is borrowed for `'1`

    error: aborting due to 2 previous errors
    ```

    </details>


  - <details><summary>In other cases sub-efficient workarounds may be available</summary>

    Such as when that `Set` would be a `Vec` instead. In that case, we can use
    indices as a poorman's self-reference, with no "official" lifetimes and thus
    Rust not complaining:

    ```rust
    # use ::std::sync::Mutex;
    #
    fn iter_locked (mutexed_vec: &'_ Mutex<Vec<i32>>)
      -> impl '_ + Iterator<Item = i32>
    {
        ::std::iter::from_fn({
            let locked_vec = mutexed_vec.lock().unwrap();
            let mut indices = 0.. locked_vec.len();
            move /* locked_vec, indices */ || {
                let i = indices.next()?;
                Some(locked_vec[i]) // copies, so OK.
            }
        })
    }
    let mutexed_elems = Mutex::new(vec![27, 42]);
    let mut iter = iter_locked(&mutexed_elems);
    assert_eq!(iter.next(), Some(27));
    assert_eq!(iter.next(), Some(42));
    assert_eq!(iter.next(), None);
    ```

    </summary>

#### But with generators this is easy:

```rust
# use ::std::{
#     collections::BTreeSet as Set,
#     sync::Mutex,
# };
use ::next_gen::prelude::*;

#[generator(yield(i32))]
fn gen_iter_locked (mutexed_elems: &'_ Mutex<Set<i32>>)
{
    let locked_elems = mutexed_elems.lock().unwrap();
    for elem in locked_elems.iter().copied() {
        yield_!(elem);
    }
}
```

_and voilà_!

That `#[generator] fn` is the key constructor for our safe self-referential
iterator!

Now, _instantiating_ an iterator off a self-referential generator has a subtle
aspect, muck alike that of polling a self-referential `Future` (that's what a
missing `Unpin` bound means): we need to get it pinned before it can be polled!

<details><summary>About pinning "before use", and the two forms of pinning</summary>

 1. Getting a `Future`:

    ```rust
    # #[cfg(any())] macro_rules! ignore {
    let future = async { ... };
    // or
    let future = some_async_fn(...);
    # }
    ```

  - Pinning an instantiated `Future` in the heap (`Box`ed):

    ```rust
    # #[cfg(any())] macro_rules! ignore {
    // Pinning it in the heap (boxed):
    let mut pinned_future = Box::pin(future)
    // or, through an extension trait (`::futures::future::FutureExt`):
    let mut pinned_future = future.boxed() // this also incidentally `dyn`-erases the future.
    # }
    ```

      - Now we can _return_ it, or poll it:

        ```rust
        # #[cfg(any())] macro_rules! ignore {
        if true {
            pinned_future.as_mut().poll(...);
        }
        // and/or return it:
        return pinned_future;
        # }
        ```

  - Pinning an instantiated `Future` in the stack (pinned to the local scope):

    ```rust
    # #[cfg(any())] macro_rules! ignore {
    use ::some_lib::some_pinning_macro as stack_pinned;
    // Pinning it in the "stack"
    stack_pinned!(mut future);
    /* the above shadows `future`, thus acting as:
    let mut future = magic::Stack::pin(future); // */

    // Let's rename it for clarity:
    let mut pinned_future = future;
    # }
    ```

      - Now we can poll it / use it within the current stack frame, **but we
        cannot return it**.

        ```rust
        # #[cfg(any())] macro_rules! ignore {
        pinned_future.as_mut().poll(...)
        # }
        ```

Well, it turns out that for generators it's similar:

 1. Once you have a `#[generator] fn` "generator constructor"

    ```rust
    use ::next_gen::prelude::*;

    #[generator(yield(u8))]
    fn foo ()
    {
        yield_!(42);
    }
    # let _ = foo;
    ```

 1. Instantiation requires pinning, and thus:

      - Stack-pinning: cheap, `no_std` compatible, usable within the same scope.
        **But it cannot be returned**.

        ```rust
        # #[cfg(any())] macro_rules! ignore {
        mk_gen!(let mut generator = foo());

        // can be used within the same scope
        assert_eq!(generator.next(), Some(42));
        assert_eq!(generator.next(), None);

        // but it can't be returned
        // return generator; /* Error, can't return borrow to local value */
        # }
        ```

      - Heap-pinning: a bit more expensive, requires an `::alloc`ator or not
        being `no_std`, **but the so-pinned generator can be returned**.

        ```rust
        # #[cfg(any())] macro_rules! ignore {
        mk_gen!(let mut generator = box foo());

        // can be used within the same scope
        if some_condition {
            assert_eq!(generator.next(), Some(42));
            assert_eq!(generator.next(), None);
        }

        // and/or it can be returned
        return generator; // OK
        # }
        ```

So, back to our example, this is what we need to do:

___

</details>

```rust
use ::next_gen::prelude::*;
# use ::std::{
#     collections::BTreeSet as Set,
#     sync::Mutex,
# };

/// We already have:
#[generator(yield(i32))]
fn gen_iter_locked (mutexed_elems: &'_ Mutex<Set<i32>>)
# {
#     let locked_elems = mutexed_elems.lock().unwrap();
#     for elem in locked_elems.iter().copied() {
#         yield_!(elem);
#     }
# }
# #[cfg(any)] macro_rules! ignore {
...
# }

/// Now let's wrap-it so that it yields a nice iterator:
fn iter_locked (mutexed_elems: &'_ Mutex<Set<i32>>)
  -> impl '_ + Iterator<Item = i32>
{
    if true {
        // One possible syntax to instantiate the generator
        mk_gen!(let generator = box gen_iter_locked(mutexed_elems));
        generator
    } else {
        // or, since we are `box`-ing, we can directly do:
        gen_iter_locked.call_boxed((mutexed_elems, ))
    }
    // : Pin<Box<impl '_ + Generator<Yield = i32>>>
    // : impl '_ + Iterator<Item = i32>
}

let mutexed_elems = Mutex::new([27, 42].iter().copied().collect::<Set<_>>());
let mut iter = iter_locked(&mutexed_elems);
assert_eq!(iter.next(), Some(27));
assert_eq!(iter.next(), Some(42));
assert_eq!(iter.next(), None);
```

  - If the `iter_locked()` function you are trying to implement is part of
    a trait definition and thus need to name the type, at which point the
    `impl '_ + Iterator…` existential syntax can be problematic, you can then
    use `dyn` instead of `impl`, at the cost of having to mention the
    `Pin<Box<>>` layer:

    ```rust
    # #[cfg(any())] macro_rules! ignore {
    // instead of
      -> impl '_ + Iterator<Item = i32>
    // write:
      -> Pin<Box<dyn '_ + Generator<Yield = i32, Return = ()>>>
    # }
    ```

    <details><summary>An example</summary>

    ```rust
    use ::next_gen::prelude::*;

    struct Once<T>(T);
    impl<T : 'static> IntoIterator for Once<T> {
        type Item = T;
        type IntoIter = Pin<Box<dyn Generator<(), Yield = T, Return = ()> + 'static>>;

        fn into_iter (self: Once<T>)
          -> Self::IntoIter
        {
            #[generator(yield(T))]
            fn once_generator<T> (value: T)
            {
                yield_!(value);
            }

            once_generator.call_boxed((self.0, ))
        }
    }
    assert_eq!(Once(42).into_iter().next(), Some(42));
    ```

    </details>

See [`Generator`](
https://docs.rs/next-gen/*/next_gen/prelude/trait.Generator.html) for more
info.

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

### `no_std` support

This crates supports `#![no_std]`. For it, just disable the default `"std"`
feature:

```toml
[dependencies]
next-gen.version = "..."
next-gen.default-features = false  # <- ADD THIS to disable `std`&`alloc` for `no_std` compat
next-gen.features = [
    "alloc",  # If your no_std platform has access to allocators.
  ## "std",  # `default-features` bundles this.
]
```

#### Idea

Since generators and coroutines rely on the same internals, one can derive a
safe implementation of generators using the `async` / `await` machinery, which
is only already in stable Rust.

A similar idea has also been implemented in <https://docs.rs/genawaiter>.
