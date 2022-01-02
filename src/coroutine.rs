use_prelude!();

/// The trait implemented by [`coroutineFn`]s **with resume arguments**.
///
/// # Example
///
/// ```rust
/// use ::next_gen::prelude::*;
///
/// fn main ()
/// {
///     #[coroutine(i32)]
///     fn coro_fn (b: bool, mut arg @ yield_: u8)
///       -> &'static str
///     {
///         assert_eq!(arg, 42);
///         arg = yield_!(1);
///         assert_eq!(arg, 27);
///         return "foo";
///     }
///
///     mk_coro!(let mut coro = coro_fn());
///
///     let mut next = |arg: u8| coro.as_mut().resume_with(arg);
///
///     match next(42) {
///         | GeneratorState::Yield(yielded) => assert_eq!(yielded, 1_i32),
///         | GeneratorState::Return(_) => unreachable!(),
///     }
///     match next(27) {
///         | GeneratorState::Yield(_) => unreachable!(),
///         | GeneratorState::Return(returned) => assert_eq!(returned, "foo"),
///     }
/// }
/// ```
///
/// # `Coroutine` _vs._ `Generator`
///
///   - A `Generator` is a `Coroutine where Resume = ()`,
///
///   - or another way to see it is that a `Coroutine` is a `Generator` which
///     has been enhanced with support for resume arguments.
///
/// We thus have the following impls:
///
/// ```rust
/// # const _: &str = stringify! {
/// impl<C : ?Sized + Coroutine<Resume = ()>> coroutine for C
///
/// #[cfg(feature = "std")]
/// impl<C : ?Sized + Coroutine> Coroutine for Pin<Box<C>>
/// impl<C : ?Sized + Coroutine> Coroutine for Pin<&mut C>
/// # };
/// ```
pub
trait Coroutine {
    /// The type of value this coroutine yields.
    ///
    /// This associated type corresponds to the `yield_!` expression and the
    /// values which are allowed to be returned each time a coroutine yields.
    /// For example an iterator-as-a-coroutine would likely have this type as
    /// `T`, the type being iterated over.
    type Yield;

    /// The type of the stateful resume argument.
    ///
    /// This associated type corresponds to the type of the value this coroutine
    /// gets when `.resumed_with()`.
    type ResumeArg;

    /// The type of value this coroutine returns.
    ///
    /// This corresponds to the type returned from a coroutine either with a
    /// `return` statement or implicitly as the last expression of a coroutine
    /// literal.
    type Return;


    /// Resumes the execution of this coroutine.
    ///
    /// This function will resume execution of the coroutine or start execution
    /// if it hasn't already. This call will return back into the coroutine's
    /// last suspension point, resuming execution from the latest `yield_!`.
    /// The coroutine will continue executing until it either yields or returns,
    /// at which point this function will return.
    ///
    /// # Return value
    ///
    /// The [`GeneratorState`] enum returned from this function indicates what
    /// state the coroutine is in upon returning.
    ///
    /// If the [`Yield`][`GeneratorState::Yield`] variant is returned then the
    /// coroutine has reached a suspension point and a value has been yielded
    /// out. Coroutines in this state are available for resumption at a later
    /// point.
    ///
    /// If [`Return`] is returned then the coroutine has completely finished
    /// with the value provided. It is invalid for the coroutine to be resumed
    /// again.
    ///
    /// # Panics
    ///
    /// This function may panic if it is called after the [`Return`] variant has
    /// been returned previously. While coroutine literals in the language are
    /// guaranteed to panic on resuming after [`Return`], this is not guaranteed
    /// for all implementations of the [`coroutine`] trait.
    ///
    /// [`Return`]: `GeneratorState::Return`
    fn resume_with (self: Pin<&'_ mut Self>, _: Self::ResumeArg)
      -> GeneratorState<Self::Yield, Self::Return>
    ;
}
