//! Internal types used by [`#[generator]`][`macro@crate::generator`]-tagged
//! functions.

use_prelude!();

pub(in crate)
mod internals {
    use super::*;

    /// The main "hack": the slot used by the `async fn` to yield items through it,
    /// at each `await` / `yield_!` point.
    ///
    /// # DO NOT USE DIRECTLY
    ///
    /// Never use or access this value directly, let the macro sugar do it.
    /// Failure to comply may jeopardize the memory safety of the program,
    /// which a failguard will detect, causing the program to abort.
    /// You have been warned.
    ///
    /// For instance, the following code leads to an abort:
    ///
    /// ```rust,no_run
    /// use ::next_gen::{__::__Internals_YieldSlot_DoNotUse__, gen_iter};
    ///
    /// async fn generator (yield_slot: __Internals_YieldSlot_DoNotUse__<'_, u8, ()>, _: ())
    ///   -> __Internals_YieldSlot_DoNotUse__<'_, u8, ()>
    /// {
    ///     yield_slot
    /// }
    ///
    /// let dangling_yield_slot = {
    ///     gen_iter!(for _ in generator() {})
    ///     // the generator() is dropped and the obtained yield_slot would thus dangle.
    ///     // this is detected by the generator destructor (guaranteed to run
    ///     // thanks to `Pin` guarantees), which then aborts the program to avoid
    ///     // unsoundness.
    /// };
    /// // <-- never reached "thanks" to the abort.
    /// let _ = dangling_yield_slot.__put(42); // write-dereference a dangling pointer !
    /// ```
    pub
    struct YieldSlot<'yield_slot, YieldedItem, ResumeArg = ()> {
        pub(in super)
        item_slot: &'yield_slot ItemSlot<YieldedItem, ResumeArg>,
    }
}
use internals::YieldSlot;

impl<YieldedItem, ResumeArg> Drop for YieldSlot<'_, YieldedItem, ResumeArg> {
    fn drop (self: &'_ mut Self)
    {
        self.item_slot.yield_slot_dropped.set(true);
    }
}

enum TransferBox<YieldedItem, ResumeArg> {
    YieldedItem(YieldedItem),
    ResumeArg(ResumeArg),
    Empty,
}

/// No pinning projection
impl<YieldedItem, ResumeArg> Unpin for TransferBox<YieldedItem, ResumeArg> {}

impl<YieldedItem, ResumeArg> TransferBox<YieldedItem, ResumeArg> {
    fn take (this: &'_ Cell<Self>)
      -> Self
    {
        this.replace(Self::Empty)
    }
}

macro_rules! misusage {( $($tt:tt)* ) => (
    ::core::format_args! {
        "Misusage of a `YieldSlot`: {}", ::core::format_args!($($tt)*),
    }
)}

struct ItemSlot<YieldedItem, ResumeArg> {
    transfer_box: Cell<TransferBox<YieldedItem, ResumeArg>>,
    yield_slot_dropped: Cell<bool>,
}

impl<'yield_slot, YieldedItem, ResumeArg>
    YieldSlot<'yield_slot, YieldedItem, ResumeArg>
{
    #[inline]
    fn new (
        item_slot: &'yield_slot ItemSlot<YieldedItem, ResumeArg>,
    ) -> YieldSlot<'yield_slot, YieldedItem, ResumeArg>
    {
        Self { item_slot }
    }

    #[doc(hidden)]
    /// Fills the slot with a value, and returns an `.await`-able to be used as
    /// yield point.
    pub
    fn __put (
        self: &'_ YieldSlot<'yield_slot, YieldedItem, ResumeArg>,
        yielded_item: YieldedItem,
    ) -> impl '_ + Future<Output = ResumeArg>
    {
        let transfer_box = &self.item_slot.transfer_box;
        let prev = transfer_box.replace(TransferBox::YieldedItem(yielded_item));
        debug_assert!(
            matches!(prev, TransferBox::Empty),
            "{}", misusage!("slot was not empty"),
        );
        poll_fn(move |_| {
            match TransferBox::take(transfer_box) {
                | yielded_item @ TransferBox::YieldedItem { .. } => {
                    transfer_box.set(yielded_item);
                    // propagate a suspension up for `Generator::resume` to
                    // handle.
                    Poll::Pending
                },
                | TransferBox::ResumeArg(resume_arg) => Poll::Ready(resume_arg),
                | TransferBox::Empty => panic!("{}", misusage!("incorrect poll")),
            }
        })
    }

    /// Takes the initial `resume_arg` off the slot.
    #[doc(hidden)]
    pub
    fn __take_initial_arg (
        self: &'_ YieldSlot<'yield_slot, YieldedItem, ResumeArg>,
    ) -> ResumeArg
    {
        match TransferBox::take(&self.item_slot.transfer_box) {
            | TransferBox::ResumeArg(resume_arg) => resume_arg,
            | _ => panic!("{}", misusage!("incorrect `take_initial_arg()`")),
        }
    }
}

/// An _instance_ of a [`#[generator]`][gen]-tagged function.
///
/// [gen]: `macro@crate::generator`
///
/// These are created in a two-step fashion:
///
///  1. First, an [`empty()`][`GeneratorFn::empty`] generator is created,
///     which is to be [pinned][`Pin`].
///
///  2. Once it is [pinned][`Pin`], it can be [`.init()`][
///     `GeneratorFn::init`]-ialized with a [`#[generator]`][gen]-tagged function.
///
/// As with any [`Generator`], for a [`GeneratorFn`] to be usable, it must have
/// been previously [`Pin`]ned:
///
///   - either in the _heap_, through [`Box::pin`];
///
///     ```rust
///     use ::next_gen::{prelude::*, generator_fn::GeneratorFn};
///
///     #[generator(yield(u32))]
///     fn countdown (mut remaining: u32)
///     {
///         while let Some(next) = remaining.checked_sub(1) {
///             yield_!(remaining);
///             remaining = next;
///         }
///     }
///
///     let generator = GeneratorFn::empty();
///     let mut generator = Box::pin(generator);
///     generator.as_mut().init(countdown, (3,));
///
///     let mut next = || generator.as_mut().resume(());
///     assert_eq!(next(), GeneratorState::Yielded(3));
///     assert_eq!(next(), GeneratorState::Yielded(2));
///     assert_eq!(next(), GeneratorState::Yielded(1));
///     assert_eq!(next(), GeneratorState::Returned(()));
///     ```
///
///   - or in the _stack_, through [`stack_pinned!`][`stack_pinned`].
///
///     ```rust
///     use ::next_gen::{prelude::*, generator_fn::GeneratorFn};
///
///     #[generator(yield(u32))]
///     fn countdown (mut remaining: u32)
///     {
///         while let Some(next) = remaining.checked_sub(1) {
///             yield_!(remaining);
///             remaining = next;
///         }
///     }
///
///     let generator = GeneratorFn::empty();
///     stack_pinned!(mut generator);
///     generator.as_mut().init(countdown, (3,));
///     let mut next = || generator.as_mut().resume(());
///     assert_eq!(next(), GeneratorState::Yielded(3));
///     assert_eq!(next(), GeneratorState::Yielded(2));
///     assert_eq!(next(), GeneratorState::Yielded(1));
///     assert_eq!(next(), GeneratorState::Returned(()));
///     ```
///
/// # `mk_gen!`
///
/// [`mk_gen!`][`mk_gen`] is a macro that reduces the boilerplate of the above
/// patterns, by performing the two step-initialization within a single macro
/// call.
///
/// # Stack _vs._ heap
///
/// Since stack-pinning prevents ever moving the generator around (duh), once
/// stack-pinned, a [`Generator`] cannot be, for instance, returned. For that,
/// [`Pin`]ning in the heap is necessary:
///
/// ```rust
/// use ::next_gen::prelude::*;
///
/// # let _ = countdown;
/// pub
/// fn countdown (count: u32)
///   -> impl Iterator<Item = u32> + 'static
/// {
///     #[generator(yield(u32))]
///     fn countdown (mut remaining: u32)
///     {
///         while let Some(next) = remaining.checked_sub(1) {
///             yield_!(remaining);
///             remaining = next;
///         }
///     }
///
///     mk_gen!(let generator = box countdown(count));
///     generator.into_iter() // A pinned generator is iterable.
/// }
/// ```
///
/// However, pinning in the stack is vastly more performant (it is zero-cost in
/// release mode), and _suffices for local iteration_. It is thus **the blessed
/// form of [`Pin`]ning**, which should be favored over `box`-ing.
///
/// ```rust
/// use ::next_gen::prelude::*;
///
/// #[generator(yield(u32))]
/// fn countdown (mut remaining: u32)
/// {
///     while let Some(next) = remaining.checked_sub(1) {
///         yield_!(remaining);
///         remaining = next;
///     }
/// }
///
/// mk_gen!(let generator = countdown(3));
/// assert_eq!(
///     generator.into_iter().collect::<Vec<_>>(),
///     [3, 2, 1],
/// );
/// ```
pub
struct GeneratorFn<YieldedItem, F : Future, ResumeArg> {
    item_slot: ItemSlot<YieldedItem, ResumeArg>,

    future: Option<F>,

    /// Once a `GeneratorFn` has been pinned, its Drop (glue) must be run
    /// before being deallocated!
    _pin_sensitive: PhantomPinned,
}

impl<YieldedItem, F : Future, ResumeArg>
    Drop
for
    GeneratorFn<YieldedItem, F, ResumeArg>
{
    fn drop (self: &'_ mut Self)
    {
        let Self { ref mut future, ref mut item_slot, .. } = *self;
        ::unwind_safe::with_state(())
            .try_eval(move |&mut ()| {
                // drop the future *in place*
                *future = None;
            })
            .finally(move |()| if item_slot.yield_slot_dropped.get_mut().not() {
                macros::abort_with_msg!("\
                    `::next_gen` fatal runtime error: \
                    a `YieldSlot` was about to dangle!\
                    \n\
                    \n\
                    This is only possible if the internals of `::next_gen` \
                    were directly (ab)used, \
                    by making a `YieldSlot` escape the `#[generator] fn`.\
                    \n\
                    Since this could lead to memory unsafety, \
                    the program will now abort.\
                ");
            })
    }
}

struct GeneratorFnPinProjected<'pin, YieldedItem, F : Future, ResumeArg> {
    item_slot: &'pin ItemSlot<YieldedItem, ResumeArg>,
    future: Pin<&'pin mut F>,
}

impl<YieldedItem, F : Future, ResumeArg>
    GeneratorFn<YieldedItem, F, ResumeArg>
{
    fn project (self: Pin<&'_ mut GeneratorFn<YieldedItem, F, ResumeArg>>)
      -> GeneratorFnPinProjected<'_, YieldedItem, F, ResumeArg>
    {
        unsafe {
            // # Safety
            //
            // This is the same as ::pin_project's .project() method:
            //
            //   - the two fields are considered transitively pinned.
            //
            //   - `Drop` does not move without calling the destructor,
            //
            //   - no packing
            let this = self.get_unchecked_mut();
            GeneratorFnPinProjected {
                item_slot: &this.item_slot,
                future: Pin::new_unchecked(
                    this.future
                        .as_mut()
                        .expect("You must init a GeneratorFn before using it!")
                ),
            }
        }
    }

    /// Reserves memory for an empty generator; to be [`Pin`]-ned afterwards.
    ///
    /// Splitting the initial creation of the `GeneratorFn` with its
    /// `init`-instantiation is needed due to the self-referential nature of
    /// a `GeneratorFn`'s constructor.
    ///
    /// Note that you do not need to call this function if using the handy
    /// [`mk_gen!`] macro.
    pub
    fn empty ()
      -> GeneratorFn<YieldedItem, F, ResumeArg>
    {
        Self {
            item_slot: ItemSlot {
                transfer_box: TransferBox::Empty.into(),
                yield_slot_dropped: false.into(),
            },
            future: None,
            _pin_sensitive: PhantomPinned,
        }
    }

    /// After [`Pin`]-ning the memory reserved by [`GeneratorFn::empty`]`()`
    /// (through [`Box::pin`] or [`stack_pinned!`]), it can be properly
    /// instanced by calling the `#[generator]`-annotated function.
    ///
    /// Splitting the initial creation of the `GeneratorFn` with its
    /// `init`-instantiation is needed due to the self-referential nature of
    /// a `GeneratorFn`'s constructor.
    ///
    /// Note that you do not need to call this function if using the handy
    /// [`mk_gen!`] macro.
    pub
    fn init<'pin, 'yield_slot, Args> (
        self: Pin<&'pin mut GeneratorFn<YieldedItem, F, ResumeArg>>,
        generator_fn: impl FnOnce(YieldSlot<'yield_slot, YieldedItem, ResumeArg>, Args) -> F,
        args: Args,
    )
    where
        YieldedItem : 'yield_slot,
        ResumeArg : 'yield_slot,
    {
        assert!(
            self.future.is_none(),
            "GeneratorFn cannot be initialized multiple times!",
        );
        unsafe {
            // # Safety
            //
            //   - This is a pinning projection except for the `future` field,
            //     to which it gets raw "unlimited" access. This is safe because
            //     the field cannot have been pinned yet (given the API).
            //
            //   - The pinning guarantee ensures the soundness of the lifetime
            //     extension: `GeneratorFn` destructor is guaranteed to run,
            //     which performs a runtime check to ensure that the
            //     `yield_slot` has been dropped. If it hasn't, the program
            //     aborts to avoid any potential unsoundness.
            let this = self.get_unchecked_mut();
            let yield_slot =
                YieldSlot::new(
                    ::core::mem::transmute::<
                        &'pin ItemSlot<YieldedItem, ResumeArg>,
                        &'yield_slot ItemSlot<YieldedItem, ResumeArg>,
                    >(
                        &this.item_slot
                    )
                )
            ;
            this.future = Some(generator_fn(yield_slot, args));
        }
    }

    /// Associated method version of [`Generator::resume`].
    #[inline]
    pub
    fn resume (
        self: Pin<&'_ mut GeneratorFn<YieldedItem, F, ResumeArg>>,
        resume_arg: ResumeArg,
    ) -> GeneratorState<YieldedItem, F::Output>
    {
        <Self as Generator<ResumeArg>>::resume(self, resume_arg)
    }
}

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
#[cfg(feature = "alloc")]
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

impl<YieldedItem, F : Future, ResumeArg>
    Generator<ResumeArg>
for
    GeneratorFn<YieldedItem, F, ResumeArg>
{
    type Yield = YieldedItem;

    type Return = F::Output;

    fn resume (
        self: Pin<&'_ mut Self>,
        resume_arg: ResumeArg,
    ) -> GeneratorState<YieldedItem, F::Output>
    {
        let this = self.project(); // panics if uninit
        let transfer_box = &this.item_slot.transfer_box;
        let prev = transfer_box.replace(TransferBox::ResumeArg(resume_arg));
        debug_assert!(
            matches!(prev, TransferBox::Empty),
            "When starting a resume, `TransferBox` is empty",
        );

        macros::create_context!(cx);
        match this.future.poll(&mut cx) {
            | Poll::Pending => {
                match TransferBox::take(transfer_box)
                {
                    | TransferBox::YieldedItem(yielded_item) => {
                        return GeneratorState::Yielded(yielded_item);
                    },
                    | _ => panic!("{}", misusage!(
                        "missing `YieldedItem` in `transfer_box`",
                    )),
                }
            },

            | Poll::Ready(value) => {
                return GeneratorState::Returned(value);
            },
        }
    }
}
