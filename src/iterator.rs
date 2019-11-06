use_prelude!();

pub
struct Iter<
    'pinned_generator,
    'item_slot : 'pinned_generator,
    Item,
    F : Future<Output = ()>,
> (
    Pin<&'pinned_generator mut Generator<'item_slot, Item, F>>,
);

impl<
    'pinned_generator,
    'item_slot : 'pinned_generator,
    Item,
    F : Future<Output = ()>,
> Iterator for Iter<'pinned_generator, 'item_slot, Item, F>
{
    type Item = Item;

    fn next (self: &'_ mut Self)
      -> Option<Item>
    {
        match self.0.as_mut().resume() {
            | GeneratorState::Yield(x) => Some(x),
            | GeneratorState::Return(()) => None,
        }
    }
}

impl<
    'pinned_generator,
    'item_slot : 'pinned_generator,
    Item,
    F : Future<Output = ()>,
> IntoIterator for Pin<&'pinned_generator mut Generator<'item_slot, Item, F>>
{
    type IntoIter = Iter<'pinned_generator, 'item_slot, Item, F>;
    type Item = Item;

    #[inline]
    fn into_iter (self: Self)
      -> Self::IntoIter
    {
        Iter(self)
    }
}
