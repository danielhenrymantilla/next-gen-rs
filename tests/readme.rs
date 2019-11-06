use ::next_gen::prelude::*;

fn main ()
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

    iter!(let iterator = range(3, 10));
    assert_eq!(
        iterator.collect::<Vec<_>>(),
        (3 .. 10).collect::<Vec<_>>(),
    );
}
