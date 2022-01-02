//! Internal types used by `#[generator]`-tagged functions.

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
    /// async fn generator (yield_slot: __Internals_YieldSlot_DoNotUse__<'_, u8>, _: ())
    ///   -> __Internals_YieldSlot_DoNotUse__<'_, u8>
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
    /// let _ = dangling_yield_slot.put(42); // write-dereference a dangling pointer !
    /// ```
    pub
    struct YieldSlot<'yield_slot, Item : 'yield_slot> {
        pub(in super)
        item_slot: Pin<&'yield_slot ItemSlot<Item>>,
    }
}
use internals::YieldSlot;

impl<Item> Drop for YieldSlot<'_, Item> {
    fn drop (self: &'_ mut Self)
    {
        self.item_slot.drop_flag.set(());
    }
}

struct ItemSlot<Item> {
    value: CellOption<Item>,
    drop_flag: CellOption<()>,
}

impl<'yield_slot, Item : 'yield_slot> YieldSlot<'yield_slot, Item> {
    #[inline]
    fn new (item_slot: Pin<&'yield_slot ItemSlot<Item>>)
      -> Self
    {
        Self { item_slot }
    }

    #[doc(hidden)]
    /// Fills the slot with a value, and returns an `.await`-able to be used as
    /// yield point.
    pub
    fn put (self: &'_ Self, value: Item)
      -> impl Future<Output = ()> + '_
    {
        let prev: Option<Item> = self.item_slot.value.set(value);
        debug_assert!(prev.is_none(), "slot was empty");
        return WaitForClear { yield_slot: self };

        /// "Dummy" `.await`-able:
        ///
        ///  1. The first time it is polled, the slot has just been filled
        ///     (_c.f._, lines above); which triggers a `Pending` yield
        ///     interruption, so that the outer thing polling it
        ///     (GeneratorFn::resume), gets to extract the value out of the
        ///     yield slot.
        ///
        ///  2. The second time it is polled, the slot is empty, so that the
        ///     generator can resume its execution to fill it again or complete
        ///     the iteration.
        struct WaitForClear<'yield_slot, Item : 'yield_slot> {
            yield_slot: &'yield_slot YieldSlot<'yield_slot, Item>,
        }

        impl<'yield_slot, Item> Future for WaitForClear<'yield_slot, Item> {
            type Output = ();

            fn poll (self: Pin<&'_ mut Self>, _: &'_ mut Context<'_>)
              -> Poll<()>
            {
                if self.yield_slot.item_slot.value.is_some() {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            }
        }
    }
}

/// An _instance_ of a `#[generator]`-tagged function.
///
/// These are created in a two-step fashion:
///
///  1. First, an [`empty()`][`GeneratorFn::empty`] generator is created,
///     which is to be [pinned][`Pin`].
///
///  2. Once it is [pinned][`Pin`], it can be [`.init()`][
///     `GeneratorFn::init`]-ialized with a `#[generator]`-tagged function.
///
/// As with any [`Generator`], for a [`GeneratorFn`] to be usable, it must have
/// been previously [`Pin`]ned:
///
///   - either in the _heap_, through [`Box::pin`];
///
///     ```rust
///     use ::next_gen::{prelude::*, generator_fn::GeneratorFn};
///
///     #[generator(u32)]
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
///     let mut next = || generator.as_mut().resume();
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
///     #[generator(u32)]
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
///     let mut next = || generator.as_mut().resume();
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
///     #[generator(u32)]
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
/// #[generator(u32)]
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
struct GeneratorFn<Item, F : Future> {
    item_slot: ItemSlot<Item>,

    future: Option<F>,

    /// Once a `GeneratorFn` has been pinned, its Drop (glue) must be run
    /// before being deallocated!
    _pin_sensitive: PhantomPinned,
}

impl<Item, F : Future> Drop
    for GeneratorFn<Item, F>
{
    fn drop (self: &'_ mut Self)
    {
        let Self { ref mut future, ref item_slot, .. } = *self;
        ::unwind_safe::with_state(())
            .try_eval(move |&mut ()| {
                // drop the future *in place*
                *future = None;
            })
            .finally(move |()| if item_slot.drop_flag.is_none() {
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

struct GeneratorPinnedFields<'pin, Item : 'pin, F : Future + 'pin> {
    item_slot: Pin<&'pin ItemSlot<Item>>,
    future: Pin<&'pin mut F>,
}

impl<Item, F : Future> GeneratorFn<Item, F> {
    fn project (self: Pin<&'_ mut Self>)
      -> GeneratorPinnedFields<'_, Item, F>
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
            GeneratorPinnedFields {
                item_slot: Pin::new_unchecked(&this.item_slot),
                future: Pin::new_unchecked(
                    this.future
                        .as_mut()
                        .expect("You must init a GeneratorFn before using it!")
                ),
            }
        }
    }

    /// Reserves memory for an empty generator.
    pub
    fn empty ()
      -> Self
    {
        Self {
            item_slot: ItemSlot {
                value: CellOption::None,
                drop_flag: CellOption::None,
            },
            future: None,
            _pin_sensitive: PhantomPinned,
        }
    }

    /// Fill the memory reserved by [`GeneratorFn::empty`]`()` with an instance
    /// of the generator function / factory.
    pub
    fn init<'pin, 'yield_slot, Args> (
        self: Pin<&'pin mut Self>,
        factory: impl FnOnce(YieldSlot<'yield_slot, Item>, Args) -> F,
        args: Args,
    )
    where
        Item : 'yield_slot,
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
                YieldSlot::new(Pin::new_unchecked(
                    ::core::mem::transmute::<
                        &'pin ItemSlot<Item>,
                        &'yield_slot ItemSlot<Item>,
                    >(
                        &this.item_slot
                    )
                ))
            ;
            this.future = Some(factory(yield_slot, args));
        }
    }

    /// Associated method version of [`Generator::resume`].
    #[inline]
    pub
    fn resume (self: Pin<&'_ mut Self>)
      -> GeneratorState<Item, F::Output>
    {
        <Self as Generator<()>>::resume(self, ())
    }
}

impl<Item, F : Future> Generator<()>
    for GeneratorFn<Item, F>
{
    type Yield = Item;

    type Return = F::Output;

    fn resume (
        self: Pin<&'_ mut Self>,
        _resume_arg: (),
    ) -> GeneratorState<Item, F::Output>
    {
        let this = self.project(); // panics if uninit
        macros::create_context!(cx);
        match this.future.poll(&mut cx) {
            | Poll::Pending => {
                let value =
                    this.item_slot
                        .value
                        .take()
                        .expect("Missing item in yield_slot!")
                ;
                GeneratorState::Yielded(value)
            },

            | Poll::Ready(value) => {
                GeneratorState::Returned(value)
            }
        }
    }
}
