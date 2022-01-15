use super::*;

/// Sugar for `Box::pin(GeneratorFn::empty()).tap_mut(|it| it.as_mut().init(…))`.
///
/// In other words,
///
/// ```rust
/// # #[cfg(any())] macro_rules! __ {
/// let gen = generator_fn.call_boxed((args, ...));
/// # }
/// ```
///
/// is the same as:
///
/// ```rust
/// # #[cfg(any())] macro_rules! __ {
/// mk_gen!(let gen = box generator_fn(args, ...));
/// # }
/// ```
///
/// ## Examples
///
/// ```rust
/// use ::next_gen::prelude::*;
/// # struct Param();
/// # struct ResumeArg();
/// # struct YieldedThing();
/// # struct ReturnValue;
/// # use ::core::mem::drop as stuff;
///
/// #[generator(yield(YieldedThing), resume(ResumeArg))]
/// fn generator_fn (param: Param)
///   -> ReturnValue
/// {
///     stuff(param);
///     let _: ResumeArg = yield_!(YieldedThing());
///     ReturnValue
/// }
///
/// let mut gen = generator_fn.call_boxed((Param(), ));
/// let _ = gen.as_mut().resume(ResumeArg());
/// ```
///
/// is thus equivalent to:
///
/// ```rust
/// use ::next_gen::prelude::*;
/// # struct Param();
/// # struct ResumeArg();
/// # struct YieldedThing();
/// # struct ReturnValue;
/// # use ::core::mem::drop as stuff;
///
/// #[generator(yield(YieldedThing), resume(ResumeArg))]
/// fn generator_fn (param: Param)
///   -> ReturnValue
/// {
///     stuff(param);
///     let _: ResumeArg = yield_!(YieldedThing());
///     ReturnValue
/// }
///
/// mk_gen!(let mut gen = box generator_fn(Param()));
/// let _ = gen.as_mut().resume(ResumeArg());
/// ```
pub trait CallBoxed<'yield_slot, YieldedItem, ResumeArg, Args> {
    ///
    type CallBoxed;

    ///
    fn call_boxed (
        self: Self,
        args: Args,
    ) -> Self::CallBoxed;
}


#[cfg(feature = "alloc")]
impl<'yield_slot, Args, Factory, F, YieldedItem, ResumeArg>
    CallBoxed<'yield_slot, YieldedItem, ResumeArg, Args>
for
    Factory
where
    YieldedItem : 'yield_slot,
    ResumeArg : 'yield_slot,
    Factory : FnOnce(YieldSlot<'yield_slot, YieldedItem, ResumeArg>, Args) -> F,
    F : Future,
{
    type CallBoxed = Pin<::alloc::boxed::Box<
        GeneratorFn<YieldedItem, F, ResumeArg>
    >>;

    fn call_boxed (
        self: Factory,
        args: Args,
    ) -> Pin<::alloc::boxed::Box<
            GeneratorFn<YieldedItem, F, ResumeArg>
        >>
    {
        let mut gen = ::alloc::boxed::Box::pin(GeneratorFn::empty());
        gen.as_mut().init(self, args);
        gen
    }
}
