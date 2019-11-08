fn main ()
{
    use ::next_gen::prelude::*;

    async fn countdown<Ret> (
        __yield_slot__: ::next_gen::YieldSlot<'_, u8>,
        (count, value): (u8, Ret),
    ) -> Ret
    {
        macro_rules! yield_ {(
            $value:expr
        ) => (
            let () = __yield_slot__.put($value).await;
        )}

        let mut current = count;
        while let Some(next) = current.checked_sub(1) {
            yield_!(current);
            current = next;
        }
        value
    }

    let generator = ::next_gen::GeneratorFn::empty();
    ::next_gen::stack_pinned!(mut generator);
    generator
        .as_mut()
        .init(countdown, (3, "Boom!"))
    ;
    let mut next = || generator.as_mut().resume();
    assert_eq!(next(), GeneratorState::Yield(3));
    assert_eq!(next(), GeneratorState::Yield(2));
    assert_eq!(next(), GeneratorState::Yield(1));
    assert_eq!(next(), GeneratorState::Return("Boom!"));
}
