//! Core generator logic: a generator is an `async fn` using a [`YieldSlot`]
//! to yield items through.
//!
//! [`YieldSlot`]: crate::generator::YieldSlot

use_prelude!();

/// The main "hack": the slot used by the `async fn` to yield items through it,
/// at each `await` / `yield_!` point.
pub
struct YieldSlot<'yield_slot, Item : 'yield_slot> (
    Pin<&'yield_slot CellOption<Item>>,
);

impl<'yield_slot, Item : 'yield_slot> YieldSlot<'yield_slot, Item> {
    #[inline]
    fn new (p: Pin<&'yield_slot CellOption<Item>>)
      -> Self
    {
        Self(p)
    }

    #[doc(hidden)]
    /// Fills the slot with a value, and returns an `.await`-able to be used as
    /// yield point.
    ///
    /// Although you can use it directly, the `yield_!()` macro takes care of
    /// that.
    pub
    fn put (self: &'_ Self, value: Item)
      -> impl Future<Output = ()> + '_
    {
        let prev: Option<Item> = self.0.set(value);
        debug_assert!(prev.is_none(), "slot was empty");
        return WaitForClear { yield_slot: self };

        /// "Dummy" `.await`-able:
        ///
        ///  1. The first time it is polled, the slot has just been filled
        ///     (_c.f._, lines above); which triggers a `Pending` yield
        ///     interruption, so that the outer thing polling it
        ///     (Generator::resume), get to extract the value out of the yield
        ///     slot.
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
                if /* while */ self.yield_slot.0.is_some() {
                    Poll::Pending
                } else {
                    Poll::Ready(())
                }
            }
        }
    }
}

/// A generator is an `async fn` function with an internal [`YieldSlot`]
pub
struct Generator<Item, F : Future> {
    yield_slot: CellOption<Item>,

    future: Option<F>,
}

struct GeneratorPinnedFields<'pin, Item : 'pin, F : Future + 'pin> {
    yield_slot: Pin<&'pin CellOption<Item>>,
    future: Pin<&'pin mut F>,
}

impl<Item, F : Future> Generator<Item, F> {
    fn project (self: Pin<&'_ mut Self>) -> GeneratorPinnedFields<'_, Item, F>
    {
        unsafe {
            // # Safety
            //
            // This is the same as ::pin_project's .project() method:
            //
            //   - No `Drop`, no packing, and the two fields are considered
            //     transitively pinned.
            let this = self.get_unchecked_mut();
            GeneratorPinnedFields {
                yield_slot: Pin::new_unchecked(&this.yield_slot),
                future: Pin::new_unchecked(
                    this.future
                        .as_mut()
                        .expect("You must init a Generator before using it!")
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
            yield_slot: CellOption::None,
            future: None,
        }
    }

    /// Fill the memory reserved by [`Generator::empty`]`()` with an instance of
    /// the generator function / factory.
    pub
    fn init<'pin, 'yield_slot, Args> (
        self: Pin<&'pin mut Self>,
        factory: impl FnOnce(YieldSlot<'yield_slot, Item>, Args) -> F,
        args: Args,
    )
    where
        Item : 'yield_slot,
    {
        assert!(self.future.is_none(),
            "Generator cannot be initialized multiple times!",
        );
        unsafe {
            // # Safety
            //
            //   - This is a pinning projection except for the `future` field,
            //     to which it gets raw "unlimited" access. This is safe because
            //     the field cannot have been pinned yet (given the API).
            //
            //   - The pinning guarantee ensures the soundness of the lifetime
            //     extension.
            let this = self.get_unchecked_mut();
            let yield_slot =
                YieldSlot::new(Pin::new_unchecked(
                    ::core::mem::transmute::<
                        &'pin CellOption<Item>,
                        &'yield_slot CellOption<Item>,
                    >(
                        &this.yield_slot
                    )
                ))
            ;
            this.future = Some(factory(yield_slot, args));
        }
    }

    /// Polls the next value of the generator.
    ///
    /// Once a [`Generator`] is completed, it should not be polled again.
    pub
    fn resume (self: Pin<&'_ mut Self>)
      -> GeneratorState<Item, F::Output>
    {
        let this = self.project(); // panics if uninit
        create_context!(cx);
        match this.future.poll(&mut cx) {
            | Poll::Pending => {
                let value =
                    this.yield_slot
                        .take()
                        .expect("Missing item in yield_slot!")
                ;
                GeneratorState::Yield(value)
            },

            | Poll::Ready(value) => {
                GeneratorState::Return(value)
            }
        }
    }
}

/// Value obtained when polling a [`Generator`]. Either it [yields][
/// `GeneratorState::Yield`] a value, or it has _just_ completed with a
/// [return value][`GeneratorState::Return`]
#[derive(
    Debug,
    Clone, Copy,
    PartialEq, Eq,
)]
pub
enum GeneratorState<Yield, Return = ()> {
    /// Value yielded by the [`Generator`], equivalent of an `Iterator`'s
    /// `Some(Item)`.
    Yield(Yield),

    /// Value _returned_ by the [`Generator`] once it has completed: contrary
    /// to an `Iterator`'s "empty" `None` value used to signal the end of the
    /// iteration, a [`Generator`] may end with a meaningful value.
    Return(Return),
}
