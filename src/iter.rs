use_prelude!();
use ::core::ops::{Deref, DerefMut};

pub
struct Iter<P> (
    Pin<P>,
)
where
    P : DerefMut,
    P::Target : Generator,
;
impl<P> Iterator for Iter<P>
where
    P : DerefMut,
    P::Target : Generator,
{
    type Item = <
        <P as Deref>::Target
        as
        Generator
    >::Yield;

    fn next (self: &'_ mut Self)
      -> Option<Self::Item>
    {
        match self.0.as_mut().resume() {
            | GeneratorState::Yield(x) => Some(x),
            | GeneratorState::Return(_) => None,
        }
    }
}

impl<'pinned_generator, Item, F : Future> IntoIterator
    for Pin<&'pinned_generator mut GeneratorFn<Item, F>>
{
    type IntoIter = Iter<&'pinned_generator mut GeneratorFn<Item, F>>;
    type Item = Item;

    #[inline]
    fn into_iter (self: Self)
      -> Self::IntoIter
    {
        Iter(self)
    }
}

#[cfg(feature = "std")]
impl<Item, F : Future> IntoIterator
    for Pin<::alloc::boxed::Box<GeneratorFn<Item, F>>>
{
    type IntoIter = Iter<::alloc::boxed::Box<GeneratorFn<Item, F>>>;
    type Item = Item;

    #[inline]
    fn into_iter (self: Self)
      -> Self::IntoIter
    {
        Iter(self)
    }
}

#[cfg(feature = "std")]
impl<Item, R> Iterator
    for Pin<::alloc::boxed::Box<dyn Generator<Yield = Item, Return = R> + '_>>
{
    type Item = Item;

    #[inline]
    fn next (self: &'_ mut Self)
      -> Option<Self::Item>
    {
        match self.as_mut().resume() {
            | GeneratorState::Yield(x) => Some(x),
            | GeneratorState::Return(_) => None,
        }
    }
}
