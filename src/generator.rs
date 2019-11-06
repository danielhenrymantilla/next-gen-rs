use_prelude!();

pub
struct ItemSlot<'item_slot, Item : 'item_slot> (
    Pin<&'item_slot CellOption<Item>>,
);

impl<'item_slot, Item : 'item_slot> From<Pin<&'item_slot CellOption<Item>>>
    for ItemSlot<'item_slot, Item>
{
    #[inline]
    fn from (p: Pin<&'item_slot CellOption<Item>>)
      -> Self
    {
        Self(p)
    }
}

impl<Item> Clone for ItemSlot<'_, Item> {
    #[inline]
    fn clone (self: &'_ Self)
      -> Self
    {
        let Self(ref inner) = *self;
        Self(*inner)
    }
}

#[pin_project]
pub
struct Generator<'item_slot, Item : 'item_slot, F : Future> {
    item_slot: ItemSlot<'item_slot, Item>,

    #[pin]
    future: F,
}

impl<'item_slot, Item : 'item_slot, F : Future> Generator<'item_slot, Item, F> {
    pub
    fn new<Args> (
        item_slot: ItemSlot<'item_slot, Item>,
        factory: impl FnOnce(Coroutine<'item_slot, Item>, Args) -> F,
        args: Args,
    ) -> Self
    {
        Self {
            future: factory(Coroutine { item_slot: item_slot.clone() }, args),
            item_slot,
        }
    }

    pub
    fn resume (self: Pin<&'_ mut Self>)
      -> GeneratorState<Item, F::Output>
    {
        let this = self.project();
        create_context!(cx);
        match this.future.poll(&mut cx) {
            | Poll::Pending => {
                let value =
                    this.item_slot
                        .0
                        .take()
                        .expect("Missing item in item_slot!")
                ;
                GeneratorState::Yield(value)
            },

            | Poll::Ready(value) => {
                GeneratorState::Return(value)
            }
        }
    }
}

#[derive(
    Debug,
    Clone, Copy,
    PartialEq, Eq,
)]
pub
enum GeneratorState<Yield, Return = ()> {
    Yield(Yield),

    Return(Return),
}

pub
struct Coroutine<'item_slot, Item : 'item_slot> {
    item_slot: ItemSlot<'item_slot, Item>,
}

impl<'slot_item, Item> Coroutine<'slot_item, Item> {
    pub
    fn _yield (self: &'_ Self, value: Item)
      -> impl Future<Output = ()> + '_
    {
        let Self { ref item_slot, .. } = *self;
        let _ = item_slot.0.set(value);
        WaitForClear { item_slot }
    }
}

struct WaitForClear<'item_slot, Item : 'item_slot> {
    item_slot: &'item_slot ItemSlot<'item_slot, Item>,
}

impl<'item_slot, Item> Future for WaitForClear<'item_slot, Item>
{
    type Output = ();

    fn poll (self: Pin<&'_ mut Self>, _: &'_ mut Context<'_>)
      -> Poll<()>
    {
        if /* while */ self.item_slot.0.is_some() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
