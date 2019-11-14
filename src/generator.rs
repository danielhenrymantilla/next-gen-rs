use_prelude!();

mod internals { use super::*;
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
/// use ::next_gen::{__Internals_YieldSlot_DoNotUse__, gen_iter};
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
#[allow(bad_style)]
pub
struct YieldSlot<'yield_slot, Item : 'yield_slot> {
    pub(in super)
    item_slot: Pin<&'yield_slot ItemSlot<Item>>,
}
} use internals::YieldSlot;

#[doc(hidden, inline)]
pub use internals::YieldSlot as __Internals_YieldSlot_DoNotUse__;

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
        ///     (GeneratorFn::resume), get to extract the value out of the yield
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
///     use ::next_gen::{prelude::*, GeneratorFn, GeneratorState};
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
///     assert_eq!(next(), GeneratorState::Yield(3));
///     assert_eq!(next(), GeneratorState::Yield(2));
///     assert_eq!(next(), GeneratorState::Yield(1));
///     assert_eq!(next(), GeneratorState::Return(()));
///     ```
///
///   - or in the _stack_, through [`stack_pinned!`][`stack_pinned`].
///
///     ```rust
///     use ::next_gen::{prelude::*, GeneratorFn, GeneratorState};
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
///     assert_eq!(next(), GeneratorState::Yield(3));
///     assert_eq!(next(), GeneratorState::Yield(2));
///     assert_eq!(next(), GeneratorState::Yield(1));
///     assert_eq!(next(), GeneratorState::Return(()));
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
}

impl<Item, F : Future> Drop for GeneratorFn<Item, F> {
    fn drop (self: &'_ mut Self)
    {
        drop(self.future.take());
        if self.item_slot.drop_flag.is_none() {
            eprintln!(concat!(
                "`::next_gen` fatal runtime error: ",
                "a `YieldSlot` was about to dangle!",
                "\n",
                "\n",
                "This is only possible if the internals of `::next_gen` were ",
                "(ab)used directly, ",
                "by making a `YieldSlot` escape the `#[generator] fn`.",
                "\n",
                "Since this could lead to memory unsafety, ",
                "the program will now abort.",
            ));
            ::std::process::abort();
        }
    }
}

struct GeneratorPinnedFields<'pin, Item : 'pin, F : Future + 'pin> {
    item_slot: Pin<&'pin ItemSlot<Item>>,
    future: Pin<&'pin mut F>,
}

impl<Item, F : Future> GeneratorFn<Item, F> {
    fn project (self: Pin<&'_ mut Self>) -> GeneratorPinnedFields<'_, Item, F>
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
        assert!(self.future.is_none(),
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
        <Self as Generator>::resume(self)
    }
}

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
/// use ::next_gen::{prelude::*, GeneratorState};
///
/// fn main ()
/// {
///     #[generator(i32)]
///     fn generator_fn () -> &'static str
///     {
///         yield_!(1);
///         return "foo"
///     }
///
///     mk_gen!(let mut generator = generator_fn());
///
///     match generator.as_mut().resume() {
///         | GeneratorState::Yield(1) => {}
///         | _ => panic!("unexpected return from resume"),
///     }
///     match generator.as_mut().resume() {
///         | GeneratorState::Return("foo") => {}
///         | _ => panic!("unexpected yield from resume"),
///     }
/// }
/// ```
pub
trait Generator {
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
    fn resume (self: Pin<&'_ mut Self>)
      -> GeneratorState<Self::Yield, Self::Return>
    ;
}

impl<Item, F : Future> Generator for GeneratorFn<Item, F> {
    type Yield = Item;

    type Return = F::Output;

    fn resume (self: Pin<&'_ mut Self>)
      -> GeneratorState<Item, F::Output>
    {
        let this = self.project(); // panics if uninit
        create_context!(cx);
        match this.future.poll(&mut cx) {
            | Poll::Pending => {
                let value =
                    this.item_slot
                        .value
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
    PartialEq, Eq,
    PartialOrd, Ord,
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
    Yield(Yield),

    /// The [`Generator`] _completed_ with a [`Return`] value.
    ///
    /// This state indicates that a [`Generator`] has finished execution with
    /// the provided value. Once a generator has returned [`Return`], it is
    /// considered a programmer error to call [`.resume()`][`Generator::resume`]
    /// again.
    ///
    /// [`Return`]: Generator::Return
    Return(Return),
}

impl<'a, G : ?Sized + 'a> Generator for Pin<&'a mut G>
where
    G : Generator,
{
    type Yield = G::Yield;
    type Return = G::Return;

    #[inline]
    fn resume (mut self: Pin<&'_ mut Pin<&'a mut G>>)
      -> GeneratorState<Self::Yield, Self::Return>
    {
        G::resume(
            Pin::<&mut G>::as_mut(&mut *self)
        )
    }
}

impl<'a, G : ?Sized + 'a> Generator for &'a mut G
where
    G : Generator + Unpin,
{
    type Yield = G::Yield;
    type Return = G::Return;

    #[inline]
    fn resume (mut self: Pin<&'_ mut &'a mut G>)
      -> GeneratorState<Self::Yield, Self::Return>
    {
        G::resume(
            Pin::<&mut G>::new(&mut *self)
        )
    }
}
