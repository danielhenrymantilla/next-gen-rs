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
        let $var = $crate::core::pin::Pin::new($var);
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

/// Instances a generator and pins it into the stack (required to be able to
/// poll it).
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
#[macro_export]
macro_rules! mk_gen {
    (@input
        let [$($mut:tt)?] $var:ident =
            $generator:tt ( $($args:expr),* $(,)? )
        $(;)?
    ) => (
        let var = $crate::generator::Generator::empty();
        $crate::stack_pinned!(mut var);
        var.as_mut()
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
