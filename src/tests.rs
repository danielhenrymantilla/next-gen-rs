use super::*;

#[test]
fn basic ()
{
    async fn generator<'foo> (co: Coroutine<'foo, u8>, _: ()) { make_yield!(co);
        yield_!(42);
        yield_!(0);
        yield_!(27);
    }

    iter!(let iterator = generator());
    assert_eq!(
        iterator.collect::<Vec<_>>(),
        [42, 0, 27],
    );
}

#[test]
fn range ()
{
    async fn range (co: Coroutine<'_, u8>, (start, end): (u8, u8)) { make_yield!(co);
        let mut current = start;
        while current < end {
            yield_!(current);
            current += 1;
        }
    }

    iter!(let iterator = range(2, 8));
    assert_eq!(
        iterator.collect::<Vec<_>>(),
        (2 .. 8).collect::<Vec<_>>(),
    );
}


