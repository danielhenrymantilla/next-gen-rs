fn main ()
{
    use ::next_gen::prelude::*;

    #[generator(u8)]
    fn countdown<Ret> (count: u8, value: Ret) -> Ret
    {
        let mut current = count;
        while let Some(next) = current.checked_sub(1) {
            yield_!(current);
            current = next;
        }
        value
    }

    mk_gen!(let mut generator = countdown(3, "Boom!"));
    let mut next = || generator.as_mut().resume();
    assert_eq!(next(), GeneratorState::Yielded(3));
    assert_eq!(next(), GeneratorState::Yielded(2));
    assert_eq!(next(), GeneratorState::Yielded(1));
    assert_eq!(next(), GeneratorState::Returned("Boom!"));
}
