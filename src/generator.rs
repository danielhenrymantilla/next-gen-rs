//! `stable` polyfill of <https://doc.rust-lang.org/1.57.0/core/ops/trait.Generator.html>.

use_prelude!();

/// The trait implemented by [`GeneratorFn`]s.
///
/// Generators, also commonly referred to as coroutines, provide an ergonomic
/// definition for iterators and other primitives, allowing to write iterators
/// and iterator adapters in a much more _imperative_ way, which may sometimes
/// improve the readability of such iterators / iterator adapters.
///
/// # Example
///
/// ```rust
/// use ::next_gen::prelude::*;
///
/// fn main ()
/// {
///     #[generator(i32)]
///     fn generator_fn ()
///       -> &'static str
///     {
///         yield_!(1);
///         return "foo";
///     }
///
///     mk_gen!(let mut generator = generator_fn());
///
///     let mut next = || generator.as_mut().resume();
///
///     match next() {
///         | GeneratorState::Yielded(yielded) => assert_eq!(yielded, 1),
///         | GeneratorState::Returned(_) => panic!("unexpected return from resume"),
///     }
///     match next() {
///         | GeneratorState::Yielded(_) => panic!("unexpected yield from resume"),
///         | GeneratorState::Returned(returned) => assert_eq!(returned, "foo"),
///     }
/// }
/// ```
///
/// # `Generator` _vs._ `Iterator`
///
///   - a `Generator` can return a non-trivial value when exhausted,
///     contrary to an `Iterator`,
///
///   - but they require to be `Pin`-ned in order to be
///     [`poll`][`Generator::resume`]ed.
///
/// We thus have the following impls:
///
/// ```rust
/// # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
/// impl<Item, F : Future> IntoIterator for Pin<&'_ mut GeneratorFn<Item, F>>
///
/// #[cfg(feature = "std")]
/// impl<Item, F : Future> IntoIterator for Pin<Box<GeneratorFn<Item, F>>>
///
/// #[cfg(feature = "std")]
/// impl<Item, R> Iterator for Pin<Box<dyn Generator<Yield = Item, Return = R> + '_>>
/// # }
/// ```
pub
trait Generator<ResumeArg> {
    /// The type of value this generator yields.
    ///
    /// This associated type corresponds to the `yield_!` expression and the
    /// values which are allowed to be returned each time a generator yields.
    /// For example an iterator-as-a-generator would likely have this type as
    /// `T`, the type being iterated over.
    type Yield;

    /// The type of value this generator returns.
    ///
    /// This corresponds to the type returned from a generator either with a
    /// `return` statement or implicitly as the last expression of a generator
    /// literal.
    type Return;


    /// Resumes the execution of this generator.
    ///
    /// This function will resume execution of the generator or start execution
    /// if it hasn't already. This call will return back into the generator's
    /// last suspension point, resuming execution from the latest `yield_!`.
    /// The generator will continue executing until it either yields or returns,
    /// at which point this function will return.
    ///
    /// # Return value
    ///
    /// The [`GeneratorState`] enum returned from this function indicates what
    /// state the generator is in upon returning.
    ///
    /// If the [`Yield`][`GeneratorState::Yield`] variant is returned then the
    /// generator has reached a suspension point and a value has been yielded
    /// out. Generators in this state are available for resumption at a later
    /// point.
    ///
    /// If [`Return`] is returned then the generator has completely finished
    /// with the value provided. It is invalid for the generator to be resumed
    /// again.
    ///
    /// # Panics
    ///
    /// This function may panic if it is called after the [`Return`] variant has
    /// been returned previously. While generator literals in the language are
    /// guaranteed to panic on resuming after [`Return`], this is not guaranteed
    /// for all implementations of the [`Generator`] trait.
    ///
    /// [`Return`]: `GeneratorState::Return`
    fn resume (
        self: Pin<&'_ mut Self>,
        resume_arg: ResumeArg,
    ) -> GeneratorState<Self::Yield, Self::Return>
    ;
}

/// Value obtained when [polling][`Generator::resume`] a [`GeneratorFn`].
///
/// This corresponds to:
///
///   - either a [suspension point][`GeneratorState::Yield`],
///
///   - or a [termination point][`GeneratorState::Return`]
#[derive(
    Debug,
    Clone, Copy,
    PartialOrd, Ord,
    PartialEq, Eq,
    Hash
)]
pub
enum GeneratorState<Yield, Return = ()> {
    /// The [`Generator`] suspended with a value.
    ///
    /// This state indicates that a [`Generator`] has been suspended, and
    /// corresponds to a `yield_!` statement. The value provided in this variant
    /// corresponds to the expression passed to `yield_!` and allows generators
    /// to provide a value each time they `yield_!`.
    Yielded(Yield),

    /// The [`Generator`] _completed_ with a [`Return`] value.
    ///
    /// This state indicates that a [`Generator`] has finished execution with
    /// the provided value. Once a generator has returned [`Return`], it is
    /// considered a programmer error to call [`.resume()`][`Generator::resume`]
    /// again.
    ///
    /// [`Return`]: Generator::Return
    Returned(Return),
}

impl<Yield> GeneratorState<Yield> {
    /// Alias for `Returned(())`.
    #[allow(nonstandard_style)]
    pub
    const Complete: Self = Self::Returned(());
}

impl<'a, ResumeArg, G : ?Sized + 'a>
    Generator<ResumeArg>
for
    Pin<&'a mut G>
where
    G : Generator<ResumeArg>,
{
    type Yield = G::Yield;
    type Return = G::Return;

    #[inline]
    fn resume (
        mut self: Pin<&'_ mut Pin<&'a mut G>>,
        arg: ResumeArg,
    ) -> GeneratorState<Self::Yield, Self::Return>
    {
        G::resume(
            Pin::<&mut G>::as_mut(&mut *self),
            arg,
        )
    }
}

impl<'a, ResumeArg, G : ?Sized + 'a>
    Generator<ResumeArg>
for
    &'a mut G
where
    G : Generator<ResumeArg> + Unpin,
{
    type Yield = G::Yield;
    type Return = G::Return;

    #[inline]
    fn resume (
        mut self: Pin<&'_ mut &'a mut G>,
        arg: ResumeArg,
    ) -> GeneratorState<Self::Yield, Self::Return>
    {
        G::resume(
            Pin::<&mut G>::new(&mut *self),
            arg,
        )
    }
}
