use_prelude!();
use ::core::ops::{Deref, DerefMut};

pub
struct IterPin<P> (
    pub(in crate) Pin<P>,
)
where
    P : DerefMut,
    P::Target : Generator<()>,
;

impl<P> Iterator for IterPin<P>
where
    P : DerefMut,
    P::Target : Generator<()>,
{
    type Item = <
        <P as Deref>::Target
        as
        Generator<()>
    >::Yield;

    fn next (self: &'_ mut Self)
      -> Option<Self::Item>
    {
        match self.0.as_mut().resume(()) {
            | GeneratorState::Yielded(x) => Some(x),
            | GeneratorState::Returned(_) => None,
        }
    }
}

impl<Item, R, F : Future<Output = R>>
    Iterator
for
    Pin<&'_ mut GeneratorFn<Item, F, ()> >
{
    type Item = Item;

    fn next (self: &'_ mut Self)
      -> Option<Self::Item>
    {
        match self.as_mut().resume(()) {
            | GeneratorState::Yielded(x) => Some(x),
            | GeneratorState::Returned(_) => None,
        }
    }
}

impl<Item, R>
    Iterator
for
    Pin<&'_ mut (
        dyn '_ + Generator<(), Yield = Item, Return = R>
    )>
{
    type Item = Item;

    fn next (self: &'_ mut Self)
      -> Option<Self::Item>
    {
        match self.as_mut().resume(()) {
            | GeneratorState::Yielded(x) => Some(x),
            | GeneratorState::Returned(_) => None,
        }
    }
}

#[cfg(feature = "alloc")]
impl<Item, R, F : Future<Output = R>>
    Iterator
for
    Pin<::alloc::boxed::Box<GeneratorFn<Item, F, ()>>>
{
    type Item = Item;

    fn next (self: &'_ mut Self)
      -> Option<Self::Item>
    {
        match self.as_mut().resume(()) {
            | GeneratorState::Yielded(x) => Some(x),
            | GeneratorState::Returned(_) => None,
        }
    }
}

#[cfg(feature = "alloc")]
impl<Item, R>
    Iterator
for
    Pin<::alloc::boxed::Box<
        dyn '_ + Generator<(), Yield = Item, Return = R>
    >>
{
    type Item = Item;

    #[inline]
    fn next (self: &'_ mut Self)
      -> Option<Self::Item>
    {
        match self.as_mut().resume(()) {
            | GeneratorState::Yielded(x) => Some(x),
            | GeneratorState::Returned(_) => None,
        }
    }
}
