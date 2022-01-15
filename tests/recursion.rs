#![cfg_attr(feature = "better-docs",
    feature(bench_black_box),
)]

use ::next_gen::prelude::*;

fn triangular (n: u64)
  -> u64
{
    #[generator(yield(u64), resume(u64))]
    fn triangular (n: u64)
      -> u64
    {
        use yield_ as recurse;
        if n == 0 {
            0
        } else {
            n + recurse!(n - 1)
        }
    }

    with_recurse(|arg| triangular.call_boxed((arg, )))(n)
}

/// where
fn with_recurse<Arg, Gen, R>(
    mut f: impl FnMut(Arg) -> Gen
) -> impl FnOnce(Arg) -> R
where
    R : Default,
    Gen : Generator<R, Yield = Arg, Return = R> + Unpin,
{
    move |arg: Arg| {
        let mut stack = Vec::new();

        let mut current = f(arg);
        let mut res = R::default();

        loop {
            match current.resume_unpin(res) {
                | GeneratorState::Yielded(arg) => {
                    stack.push(current);
                    current = f(arg);
                    res = R::default();
                }
                | GeneratorState::Returned(real_res) => {
                    match stack.pop() {
                        | None => return real_res,
                        | Some(top) => {
                            current = top;
                            res = real_res;
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn main ()
{
    assert_eq!(
        140737496743936,
        triangular(1_u64 << 24),
    );
}

#[cfg(feature = "better-docs")]
#[test]
#[ignore] // Naïve recursion overflows the stack.
fn naive ()
{
    fn triangular (n: u64)
      -> u64
    {
        if n == 0 {
            0
        } else {
            n + triangular(::core::hint::black_box(n - 1))
        }
    }

    assert_eq!(
        140737496743936,
        triangular(1_u64 << 24),
    )
}
