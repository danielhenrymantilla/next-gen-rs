/// Pins a local to the stack.
///
/// This is used by [`mk_gen`]`!` to get a pollable generator without going
/// through a heap allocation (as [`Box::pin`] would do).
#[macro_export]
macro_rules! stack_pinned {
    (
        $var:ident
    ) => (
        let (ref $var,) = ($var,);
        let $var = unsafe {
            /// # Safety
            ///
            ///   - This is pin_utils' `pin_mut!` macro: the shadowing ensures
            ///     there is no longer access to the original stack variable,
            ///     which is thus impossible to move or forget.
            extern {}
            $crate::core::pin::Pin::new_unchecked($var)
        };
    );

    (
        mut $var:ident
    ) => (
        let (ref mut $var,) = ($var,);
        #[allow(unused_mut)]
        let mut $var = unsafe {
            /// # Safety
            ///
            ///   - This is pin_utils' `pin_mut!` macro: the shadowing ensures
            ///     there is no longer access to the original stack variable,
            ///     which is thus impossible to move or forget.
            extern {}
            $crate::core::pin::Pin::new_unchecked($var)
        };
    );
}

/// Instances a generator and pins it (required to be able to poll it).
///
/// By default it pins to the stack, but by using `box` it can pin to the heap
/// (necessary when wanting to return the generator itself).
///
/// # Usage
///
/// > `mk_gen!(let $(mut)? <varname> = $(box)? <generator fn> (<args>));`
///
/// # Example
///
/// ```rust
/// use ::next_gen::prelude::*;
///
/// #[generator(bool)]
/// fn toggler (initial: bool)
/// {
///     use ::core::ops::Not;
///
///     let mut current = initial;
///     loop {
///         yield_!(current);
///         current = current.not();
///     }
/// }
///
/// mk_gen!(let generator = toggler(true));
/// // a generator is not an iterator but an iterable:
/// let mut iterator = generator.into_iter();
///
/// assert_eq!(iterator.next(), Some(true));
/// assert_eq!(iterator.next(), Some(false));
/// assert_eq!(iterator.next(), Some(true));
/// assert_eq!(iterator.next(), Some(false));
/// assert_eq!(iterator.take(10_000).count(), 10_000);
/// ```
///
/// See [`GeneratorFn`] for more examples.
#[macro_export]
macro_rules! mk_gen {
    (@input
        let [$($mut:tt)?] $var:ident =
            box $generator:tt ( $($args:expr),* $(,)? )
        $(;)?
    ) => (
        let mut var = $crate::alloc::boxed::Box::pin(
            $crate::GeneratorFn::empty()
        );
        var .as_mut()
            .init(
                $generator,
                ($($args, )*),
            )
        ;
        let $($mut)? $var = var;
    );

    (@input
        let [$($mut:tt)?] $var:ident =
            $generator:tt ( $($args:expr),* $(,)? )
        $(;)?
    ) => (
        let var = $crate::GeneratorFn::empty();
        $crate::stack_pinned!(mut var);
        var .as_mut()
            .init(
                $generator,
                ($($args, )*),
            )
        ;
        let $($mut)? $var = var;
    );

    (
        let mut $($tt:tt)*
    ) => (
        $crate::mk_gen!(@input let [mut] $($tt)*)
    );

    (
        let $($tt:tt)*
    ) => (
        $crate::mk_gen!(@input let [] $($tt)*)
    );
}

/// Emulate a `for`-loop iteration over a generator. The call itself evaluates
/// to the [`Return`][`crate::Generator::Return`] value of the [`Generator`][
/// `crate::Generator`].
///
/// # Example
///
/// ```rust
/// use ::next_gen::prelude::*;
///
/// type Question = &'static str;
/// type Answer = i32;
///
/// #[generator(Question)]
/// fn answer () -> Answer
/// {
///     yield_!("What is the answer to life, the universe and everything?");
///     42
/// }
///
/// let ret = gen_iter!(
///     for question in answer() {
///         assert_eq!(
///             question,
///             "What is the answer to life, the universe and everything?",
///         );
///     }
/// );
/// assert_eq!(ret, 42);
///
/// // You can also give it an already instanced generator:
///
/// mk_gen!(let mut generator = answer());
/// assert_eq!(
///     generator.as_mut().resume(),
///     ::next_gen::GeneratorState::Yield(
///         "What is the answer to life, the universe and everything?"
///     ),
/// );
///
/// let ret = gen_iter!(
///     for _ in generator {
///         unreachable!();
///     }
/// );
/// assert_eq!(ret, 42);
/// ```
/// ___
///
/// Note that you do not need this macro when you don't care about the return
/// value:
///
/// ```rust
/// use ::next_gen::prelude::*;
///
/// type Question = &'static str;
/// type Answer = i32;
///
/// #[generator(Question)]
/// fn answer () -> Answer
/// {
///     yield_!("What is the answer to life, the universe and everything?");
///     42
/// }
///
/// mk_gen!(let generator = answer());
///
/// for question in generator {
///     assert_eq!(
///         question,
///         "What is the answer to life, the universe and everything?",
///     );
/// }
/// ```
#[macro_export]
macro_rules! gen_iter {
    (
        for $pat:pat in $generator:tt ($($args:expr),* $(,)?) $block:block
    ) => ({
        $crate::mk_gen!(let generator = $generator ($($args),*));
        $crate::gen_iter!(
            for $pat in generator $block
        )
    });

    (
        for $pat:pat in $generator:tt $block:block
    ) => (match $generator { mut generator => {
        use $crate::{
            core::pin::Pin,
            Generator,
            GeneratorState,
        };
        let mut resume_generator = || -> GeneratorState<_, _> {
            Generator::resume(
                Pin::as_mut(&mut generator)
            )
        };
        loop {
            match resume_generator() {
                | GeneratorState::Yield($pat) => $block,
                | GeneratorState::Return(ret) => break ret,
            }
        }
    }});
}
