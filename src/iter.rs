use_prelude!();

pub
struct Iter<'pinned_generator, Item, F : Future> (
    Pin<&'pinned_generator mut Generator<Item, F>>,
);

impl<'pinned_generator, Item, F : Future> Iterator
    for Iter<'pinned_generator, Item, F>
{
    type Item = Item;

    fn next (self: &'_ mut Self)
      -> Option<Item>
    {
        match self.0.as_mut().resume() {
            | GeneratorState::Yield(x) => Some(x),
            | GeneratorState::Return(_) => None,
        }
    }
}

impl<'pinned_generator, Item, F : Future> IntoIterator
    for Pin<&'pinned_generator mut Generator<Item, F>>
{
    type IntoIter = Iter<'pinned_generator, Item, F>;
    type Item = Item;

    #[inline]
    fn into_iter (self: Self)
      -> Self::IntoIter
    {
        Iter(self)
    }
}
