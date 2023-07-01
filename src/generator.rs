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
///     #[generator(yield(i32))]
///     fn generator_fn ()
///       -> &'static str
///     {
///         yield_!(1);
///         return "foo";
///     }
///
///     mk_gen!(let mut generator = generator_fn());
///
///     let mut next = || generator.as_mut().resume(());
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
/// # #[cfg(any())] macro_rules! __ {
/// impl<Item, R, F>
///     Iterator
/// for
///     Pin<&'_ mut (
///         GeneratorFn<Item, F, ()>
///     )>
/// where
///     GeneratorFn<Item, F, ()> : Generator<(), Yield = Item, Return = R>,
///
/// impl<Item, R>
///     Iterator
/// for
///     Pin<&'_ mut (
///         dyn '_ + Generator<(), Yield = Item, Return = R>
///     )>
/// # }
/// ```
///
/// and, when under `#[cfg(feature = "alloc")]`, we also have:
///
/// ```rust
/// # #[cfg(any())] macro_rules! __ {
/// impl<Item, R, F>
///     Iterator
/// for
///     Pin<Box<
///         GeneratorFn<Item, F, ()>,
///     >>
/// where
///     GeneratorFn<Item, F, ()> : Generator<(), Yield = Item, Return = R>,
///
/// impl<Item, R>
///     Iterator
/// for
///     Pin<Box<
///         dyn '_ + Generator<(), Yield = Item, Return = R>,
///     >>
/// # }
/// ```
///
///   - #### A remark regarding the lack of blanket impl and coherence
///
///     Since `{,Into}Iterator` is defined in `::core`, and this definition of
///     [`Generator`] is a third-party library one, for coherence reasons, it is
///     not possible to implement `{,Into}Iterator` for all the `impl
///     Generator`s.
///
///     That being said, the above impls do cover the `dyn Generator` and
///     `GeneratorFn` cases, which ought to cover all the
///     `#[generator] fn`-originating generator instances.
///
///     Should such impls not be enough, there is always the
///     [`.boxed_gen_into_iter()`][`GeneratorExt::boxed_gen_into_iter`] and
///     [`.gen_into_iter()`][`GeneratorExt::gen_into_iter`] methods
///     to convert _any_ pinned generator (provided `ResumeArg = ()`) into an
///     interator.
#[cfg_attr(feature = "nightly", doc(notable_trait))]
pub
trait Generator<ResumeArg = ()> {
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
    /// If the [`Yield`][`GeneratorState::Yielded`] variant is returned then the
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
    /// [`Return`]: `GeneratorState::Returned`
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
///   - either a [suspension point][`GeneratorState::Yielded`],
///
///   - or a [termination point][`GeneratorState::Returned`]
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

impl<Yield> GeneratorState<Yield, ()> {
    /// Alias for `Returned(())`.
    #[allow(nonstandard_style)]
    pub
    const Complete: Self = Self::Returned(());
}

// # TRANSITIVE IMPLS
// ## `?Unpin`
impl<ResumeArg, G : ?Sized>
    Generator<ResumeArg>
for
    Pin<&'_ mut G>
where
    G : Generator<ResumeArg>,
{
    transitive_impl_deferring_to!(|self| (*self).as_mut());
}
#[cfg(feature = "alloc")]
impl<ResumeArg, G : ?Sized>
    Generator<ResumeArg>
for
    Pin<::alloc::boxed::Box<G>>
where
    G : Generator<ResumeArg>,
{
    transitive_impl_deferring_to!(|self| (*self).as_mut());
}

// ## `Unpin`
impl<ResumeArg, G : ?Sized>
    Generator<ResumeArg>
for
    &'_ mut G
where
    G : Generator<ResumeArg> + Unpin,
{
    transitive_impl_deferring_to!(|self| Pin::new(&mut **self));
}
#[cfg(feature = "alloc")]
impl<ResumeArg, G : ?Sized>
    Generator<ResumeArg>
for
    ::alloc::boxed::Box<G>
where
    G : Generator<ResumeArg> + Unpin,
{
    transitive_impl_deferring_to!(|self| Pin::new(&mut **self));
}

// where:
macro_rules! transitive_impl_deferring_to {(
    |$self:tt| $expr:expr $(,)?
) => (
    type Yield = G::Yield;
    type Return = G::Return;

    #[inline]
    fn resume (
        mut $self: Pin<&'_ mut Self>,
        arg: ResumeArg,
    ) -> GeneratorState<Self::Yield, Self::Return>
    {
        <G as Generator<ResumeArg>>::resume($expr, arg)
    }
)} use transitive_impl_deferring_to;

/// Extension trait with some convenience methods for [`Generator`]s.
pub
trait GeneratorExt<ResumeArg>
:
    Generator<ResumeArg> +
{
    /// Same as [`.resume()`][`Generator::resume`], but with a `&mut Self`
    /// receiver rather than a `Pin<&mut Self>` one, for convenience, thanks to
    /// the `Unpin` bound.
    ///
    /// Basically `g.resume_unpin(arg)` is sugar for
    /// `Pin::new(&mut g).resume(arg)` (or also `g.as_mut().resume(arg)`, when
    /// `g` is already a `Pin`-wrapped pointer).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ::next_gen::prelude::*;
    /// #
    /// fn example<ResumeArg, G : ?Sized> (
    ///     g: &'_ mut G,
    ///     resume_arg: ResumeArg,
    /// ) -> GeneratorState<G::Yield, G::Return>
    /// where
    ///     G : Generator<ResumeArg> + Unpin,
    /// {
    ///     g.resume_unpin(resume_arg)
    /// }
    /// ```
    #[inline]
    fn resume_unpin (
        self: &'_ mut Self,
        resume_arg: ResumeArg,
    ) -> GeneratorState<Self::Yield, Self::Return>
    where
        Self : Unpin,
    {
        Pin::new(self).resume(resume_arg)
    }

    /// Convenience method to convert _any_ (boxed) generator into an
    /// iterator.
    ///
    ///   - (provided `ResumeArg = ()`).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ::next_gen::prelude::*;
    /// #
    /// fn example<'g, T : 'g, G : ?Sized + 'g> (
    ///     g: Pin<Box<G>>,
    /// ) -> impl 'g + Iterator<Item = T>
    /// where
    ///     G : Generator<(), Yield = T>,
    /// {
    ///     g.boxed_gen_into_iter()
    /// }
    /// ```
    #[cfg(feature = "alloc")]
    #[inline]
    fn boxed_gen_into_iter (
        self: Pin<::alloc::boxed::Box<Self>>,
    ) -> crate::iter::IterPin<
            ::alloc::boxed::Box<Self>,
        >
    where
        Self : Generator<()>,
        crate::iter::IterPin< ::alloc::boxed::Box<Self> >
            : Iterator<Item = <Self as Generator<()>>::Yield>
        ,
    {
        crate::iter::IterPin(self)
    }

    /// Convenience method to convert _any_ borring pinned generator into an
    /// iterator.
    ///
    ///   - (provided `ResumeArg = ()`).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ::next_gen::prelude::*;
    /// #
    /// fn example<'g, T : 'g, G : ?Sized> (
    ///     g: Pin<&'g mut G>,
    /// ) -> impl 'g + Iterator<Item = T>
    /// where
    ///     G : Generator<(), Yield = T>,
    /// {
    ///     g.gen_into_iter()
    /// }
    /// ```
    fn gen_into_iter<'lt> (
        self: Pin<&'lt mut Self>,
    ) -> crate::iter::IterPin<
            &'lt mut Self,
        >
    where
        Self : Generator<()>,
        crate::iter::IterPin< &'lt mut Self >
            : Iterator<Item = <Self as Generator<()>>::Yield>
        ,
    {
        crate::iter::IterPin(self)
    }
}

impl<ResumeArg, G : ?Sized>
    GeneratorExt<ResumeArg>
for
    G
where
    G : Generator<ResumeArg>,
{}

/// Ensure that the [`Send`] trait is only implemented for [`Generator`]s whose
/// locals also implement [`Send`].
/// ```compile_fail
/// use ::next_gen::prelude::*;
/// use ::std::cell::Cell;
/// use ::std::iter::FromIterator;
///
/// fn non_send ()
/// {
///     #[generator(yield(u8))]
///     fn range (start: u8, end: u8)
///     {
///         let mut current: Cell<u8> = start.into();
///         while current.get() < end {
///             yield_!(current.get());
///             *current.get_mut() += 1;
///         }
///     }
///
///     mk_gen!(let mut generator = box range(1, 8));
///     assert_eq!(generator.as_mut().resume(()), GeneratorState::Yielded(1));
///     std::thread::spawn(move || {
///       assert_eq!(
///         generator.into_iter().collect::<Vec<_>>(),
///         Vec::from_iter(2 .. 8),
///       )
///     }).join().unwrap();
/// }
/// ```
fn _compile_error_test() {}
