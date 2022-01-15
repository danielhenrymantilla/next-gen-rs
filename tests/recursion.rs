#![cfg_attr(feature = "nightly",
    feature(bench_black_box),
)]

const N: u64 = 1_u64 << 16;
const OUTPUT: u64 = 2147516416;

#[test]
#[ignore]
fn naive_stack_state ()
{
    assert_eq!(
        naive_stack_state::triangular(N),
        OUTPUT,
    );
}

#[test]
fn auto_heap_state ()
{
    assert_eq!(
        auto_heap_state::triangular(N),
        OUTPUT,
    );
}

mod naive_stack_state {
    pub
    fn triangular (n: u64)
      -> u64
    {
        if n == 0 {
            0
        } else {
            let recurse_arg = n - 1;
            #[cfg(feature = "nightly")]
            let recurse_arg = ::core::hint::black_box(recurse_arg);
            n + triangular(recurse_arg)
        }
    }
}

mod auto_heap_state {
    use ::next_gen::prelude::*;

    /// A recursive computation can be seen as a "suspensible coroutine",
    /// whereby, when needing to "compute-recurse" into (smaller) parameters,
    /// that current computation just suspends and yields the new parameter
    /// for which it requests a computation.
    ///
    /// The driver / "executor", thus starts with the initial argument, and
    /// polls the suspensible coroutine until reaching a suspension point.
    ///
    /// Such suspension point gives the driver a new computation it needs to
    /// perform (updates `arg`), and a new "customer" waiting for that new
    /// result: that suspended computation. These stack onto each other as
    /// we recurse, and when the innermost computation _completes_ / _returns_
    /// rather than yield-enqueuing a new one, we can then feed that result to
    /// the top-most suspended computation, _resuming_ it.
    fn drive_recursion<Arg, Gen, R>(
        arg: Arg,
        mut start_computing: impl FnMut(Arg) -> Gen,
    ) -> R
    where
        Gen : Generator<R, Yield = Arg, Return = R> + Unpin,
        R : Default, // to feed the initial dummies.
    {
        // This is the "recursive state stack", when you think about this,
        // and with this approach we automagically get it heap-allocated
        // (the `Pin<Box<GeneratorFn…>>` state machines are the main things
        // heap-allocating the "recursively captured local state".
        // This vec is just storing these `Pin<Box<…>>` things, to avoid
        // stack-allocating those (which naively recursing within this very body
        // would achieve).
        let mut suspended_computations = Vec::<Gen>::new();

        let mut last_suspended_computation = start_computing(arg);
        let mut computation_result = R::default(); // start with a dummy


        loop {
            match last_suspended_computation.resume_unpin(computation_result) {
                // We reached `return`: completion of the current computation.
                | GeneratorState::Returned(computation_result_) => {
                    match suspended_computations.pop() {
                        // If it was the outer-most computation, we've finished.
                        | None => return computation_result_,
                        // Otherwise, feed the current result to the outer
                        // computation that had previously yield-requested the
                        // current computation.
                        | Some(suspended_computation) => {
                            last_suspended_computation = suspended_computation;
                            computation_result = computation_result_;
                        },
                    }
                },
                // We need to "compute-recurse" ourselves with this new `arg`
                | GeneratorState::Yielded(arg) => {
                    suspended_computations.push(last_suspended_computation);
                    last_suspended_computation = start_computing(arg);
                    computation_result = R::default();
                },
            }
        }
    }

    pub
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

        drive_recursion(n, |n| triangular.call_boxed((n, )))
    }
}
