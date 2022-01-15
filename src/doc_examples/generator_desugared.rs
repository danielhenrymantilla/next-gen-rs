fn main ()
{
    use ::next_gen::prelude::*;

    async fn countdown<Ret>(
        __yield_slot__: ::next_gen::__::__Internals_YieldSlot_DoNotUse__<'_, u8>,
        (count, value): (u8, Ret),
    ) -> Ret {
        macro_rules! yield_ {( $value:expr $(,)? ) => (
            __yield_slot__.__put($value).await
        )}
        let _ = __yield_slot__.__take_initial_arg();
        {
            let mut current = count;
            while let Some(next) = current.checked_sub(1) {
                yield_!(current);
                current = next;
            }
            value
        }
    }

    let generator = ::next_gen::generator_fn::GeneratorFn::empty();
    ::next_gen::stack_pinned!(mut generator);
    generator
        .as_mut()
        .init(countdown, (3, "Boom!"))
    ;
    let mut next = || generator.as_mut().resume(());
    assert_eq!(next(), GeneratorState::Yielded(3));
    assert_eq!(next(), GeneratorState::Yielded(2));
    assert_eq!(next(), GeneratorState::Yielded(1));
    assert_eq!(next(), GeneratorState::Returned("Boom!"));
}
