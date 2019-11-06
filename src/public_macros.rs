#[macro_export]
macro_rules! make_yield {
    (
        @with_dollar![$dol:tt]
        $co:expr
    ) => (
        macro_rules! yield_ {(
            $value:expr
        ) => (
            $co._yield($value).await
        )}
    );

    (
        $co:expr
    ) => (
        $crate::make_yield!(
            @with_dollar![$]
            $co
        )
    )
}

#[macro_export]
macro_rules! iter {(
    let $var:ident = $generator:tt ( $($args:expr),* $(,)?) $(;)?
) => (
    let slot = $crate::prelude::CellOption::None;
    let slot = unsafe {
        $crate::core::pin::Pin::new_unchecked(&slot)
    };
    let mut $var = $crate::Generator::<'_>::new(slot.into(), $generator, ($($args, )*));
    let $var = unsafe {
        $crate::core::pin::Pin::new_unchecked(&mut $var)
    };
    let $var = $crate::core::iter::IntoIterator::into_iter($var);
)}
