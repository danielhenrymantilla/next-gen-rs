use {
    ::core::{
        iter::FromIterator,
    },
    ::std::prelude::v1::*,
    crate::{
        __::__Internals_YieldSlot_DoNotUse__ as YieldSlot,
    },
    super::*,
};

macro_rules! assert_it_eq {(
    $left:expr, $right:expr $(, $($msg:expr $(,)?)?)?
) => (
    assert_eq!(
        $left.into_iter().collect::<Vec<_>>(),
        $right,
        $($($msg ,)?)?
    )
)}

macro_rules! make_yield {
    (
        @with_dollar![$dol:tt]
        $co:expr
    ) => (
        macro_rules! yield_ {(
            $dol value:expr
        ) => (
            $co.put($dol value).await
        )}
    );

    (
        $co:expr
    ) => (
        make_yield!(
            @with_dollar![$]
            $co
        )
    )
}

#[test]
fn basic ()
{
    async fn generator<'foo> (out: YieldSlot<'foo, u8>, _: ()) {
        make_yield!(out);

        yield_!(42);
        yield_!(0);
        yield_!(27);
    }

    mk_gen!(let generator = generator());
    assert_it_eq!(
        generator,
        [42, 0, 27],
    );
}

#[test]
fn range ()
{
    async fn range (out: YieldSlot<'_, u8>, (start, end): (u8, u8)) {
        make_yield!(out);

        let mut current = start;
        while current < end {
            yield_!(current);
            current += 1;
        }
    }

    mk_gen!(let generator = range(2, 8));
    assert_it_eq!(
        generator,
        Vec::from_iter(2 .. 8),
    );
}

mod proc_macros {
    use super::*;
    use ::next_gen_proc_macros::generator;

    #[test]
    fn range ()
    {
        #[generator(u8)]
        fn range (start: u8, end: u8)
        {
            let mut current = start;
            while current < end {
                yield_!(current);
                current += 1;
            }
        }

        mk_gen!(let generator = range(2, 8));
        assert_it_eq!(
            generator,
            Vec::from_iter(2 .. 8),
        );
    }

    #[test]
    fn gen_iter ()
    {
        type Question = &'static str;
        type Answer = i32;

        #[generator(Question)]
        fn answer () -> Answer
        {
            yield_!("What is the answer to life, the universe and everything?");
            42
        }

        let ret = gen_iter!(
            for question in answer() {
                assert_eq!(
                    question,
                    "What is the answer to life, the universe and everything?",
                );
            }
        );
        assert_eq!(ret, 42);
    }

    mod adaptors {
        use super::*;

        #[generator(T)]
        fn filter<T> (
            mut predicate: impl FnMut(&T) -> bool,
            iterable: impl IntoIterator<Item = T>,
        )
        {
            for element in iterable {
                if predicate(&element) {
                    yield_!(element);
                }
            }
        }

        #[generator(U)]
        fn map<T, U> (
            mut f: impl FnMut(T) -> U,
            iterable: impl IntoIterator<Item = T>,
        )
        {
            for element in iterable {
                yield_!(f(element));
            }
        }

        #[generator(u8)]
        fn range (start: u8, end: u8)
        {
            let mut current = start;
            while current < end {
                yield_!(current);
                current += 1;
            }
        }

        #[test]
        fn filter_range ()
        {

            mk_gen!(let iterator = range(2, 7));
            mk_gen!(let iterator = filter(|x| x % 2 == 0, iterator));
            assert_it_eq!(
                iterator,
                [2, 4, 6],
            );
        }

        #[test]
        fn filter_map_range ()
        {

            mk_gen!(let iterator = range(2, 7));
            mk_gen!(let iterator = filter(|x| x % 2 == 0, iterator));
            mk_gen!(let iterator = map(|x| x * x, iterator));
            assert_it_eq!(
                iterator,
                [4, 16, 36],
            );
        }
    }

    #[test]
    fn return_iterator_with_concrete_dyn_type ()
    {
        trait Countdown {
            type Iter : Iterator<Item = u8>;
            fn countdown (self: &'_ Self) -> Self::Iter;
        }
        struct CountdownFrom(u8);
        enum Void {} type None = Option<Void>;
        impl Countdown for CountdownFrom {
            type Iter = Pin<Box<dyn Generator<(), Yield = u8, Return = None>>>;
            fn countdown (self: &'_ Self)
              -> Self::Iter
            {
                #[generator(u8)]
                fn countdown (from: u8) -> Option<Void>
                {
                    let mut current = from;
                    loop {
                        yield_!(current);
                        current = current.checked_sub(1)?;
                    }
                }
                mk_gen!(let gen = box countdown(self.0));
                gen
            }
        }
        assert_it_eq!(
            CountdownFrom(3).countdown(),
            [3, 2, 1, 0],
        );
    }
}
