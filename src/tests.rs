#![allow(unused_imports)]
use {
    ::core::{
        iter::FromIterator,
    },
    ::std::{*, panic, prelude::v1::*},
    crate::{
        __::__Internals_YieldSlot_DoNotUse__ as YieldSlot,
    },
    super::*,
};

#[test]
fn basic ()
{
    async
    fn generator<'foo> (out: YieldSlot<'foo, u8, ()>, _: ())
    {
        make_yield!(out);
        let _ = out.__take_initial_arg();

        let _: () = yield_!(42);
        let _: () = yield_!(0);
        let _: () = yield_!(27);
    }

    mk_gen!(let generator = generator());
    assert_it_eq!(
        generator,
        [42, 0, 27],
    );
}

#[test]
fn resume_args ()
{
    async
    fn generator<'foo> (
        out: YieldSlot<'foo, u8, i32>,
        (): (),
    ) -> Vec<i32>
    {
        make_yield!(out);
        let mut resume_args = vec![];

        let mut arg = out.__take_initial_arg();

        while arg != 0 {
            resume_args.push(arg);
            arg = yield_!(arg as _);
        }

        resume_args
    }

    mk_gen!(let mut generator = generator());
    let mut resume = |arg| match generator.as_mut().resume(arg) {
        | GeneratorState::Yielded(yielded_value) => {
            assert_eq!(yielded_value as i32, arg);
            None
        },
        | GeneratorState::Returned(ret) => {
            assert_eq!(arg, 0);
            Some(ret)
        },
    };

    resume(12);
    resume(17);
    resume(47);
    assert_eq!(resume(0).unwrap(), vec![12, 17, 47]);
}

#[test]
fn range ()
{
    async
    fn range (out: YieldSlot<'_, u8, ()>, (start, end): (u8, u8))
    {
        make_yield!(out);
        let _ = out.__take_initial_arg();

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
    use super::{*, panic};
    use ::next_gen_proc_macros::generator;

    #[test]
    fn resume_args ()
    {
        #[generator(yield(u8), resume(i32) as mut arg)]
        fn generator<'foo> ()
          -> Vec<i32>
        {
            let mut resume_args = vec![];
            while arg != 0 {
                resume_args.push(arg);
                arg = yield_!(arg as _);
            }

            resume_args
        }

        mk_gen!(let mut generator = generator());
        let mut resume = |arg| match generator.as_mut().resume(arg) {
            | GeneratorState::Yielded(yielded_value) => {
                assert_eq!(yielded_value as i32, arg);
                None
            },
            | GeneratorState::Returned(ret) => {
                assert_eq!(arg, 0);
                Some(ret)
            },
        };

        resume(12);
        resume(17);
        resume(47);
        assert_eq!(resume(0).unwrap(), vec![12, 17, 47]);
    }

    #[test]
    fn range ()
    {
        #[generator(yield(u8))]
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

        #[generator(yield(Question))]
        fn answer ()
          -> Answer
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
        use super::{*, panic};

        #[generator(yield(T))]
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

        #[generator(yield(U))]
        fn map<T, U> (
            mut f: impl FnMut(T) -> U,
            iterable: impl IntoIterator<Item = T>,
        )
        {
            for element in iterable {
                yield_!(f(element));
            }
        }

        #[generator(yield(u8))]
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
        enum Void {}
        type None = Option<Void>;

        trait Countdown {
            type Iter : Iterator<Item = u8>;
            fn countdown (self: &'_ Self)
              -> Self::Iter
            ;
        }

        struct CountdownFrom(u8);
        impl Countdown for CountdownFrom {
            type Iter = Pin<Box<
                dyn Generator<(), Yield = u8, Return = None>
            >>;

            fn countdown (self: &'_ CountdownFrom)
              -> Pin<Box<
                    dyn Generator<(), Yield = u8, Return = None>
                >>
            {
                #[generator(yield(u8))]
                fn countdown (from: u8)
                  -> None
                {
                    let mut current = from;
                    loop {
                        yield_!(current);
                        current = current.checked_sub(1)?;
                    }
                }
                mk_gen!(let countdown = box countdown(self.0));
                countdown
            }
        }
        assert_it_eq!(
            CountdownFrom(3).countdown(),
            [3, 2, 1, 0],
        );
    }

    /// Ensure that the [`Send`] trait is implemented when the [`Generator`]s
    /// does not use any non-[`Send`] types.
    #[test]
    fn send() {
        #[generator(yield(u8))]
        fn range(start: u8, end: u8) {
            let mut current = start;
            while current < end {
                yield_!(current);
                current += 1;
            }
        }

        mk_gen!(let mut generator = box range(1, 8));
        assert_eq!(generator.as_mut().resume(()), GeneratorState::Yielded(1));
        std::thread::spawn(move || {
            assert_it_eq!(generator, Vec::from_iter(2..8),);
        })
        .join()
        .unwrap();
    }

    /// Ensure that the [`Send`] trait is implemented when the [`Generator`]s
    /// arguments are not [`Send`], as long as the arguments are not held across
    /// [`yield_`].
    #[test]
    fn non_send_arg() {
      #[generator(yield(u8))]
      fn range(start: Cell<u8>, end: u8) {
          let mut current: u8 = start.get();
          drop(start);
          while current < end {
              yield_!(current);
              current += 1;
          }
      }
      mk_gen!(let mut generator = box range(Cell::new(1), 8));
      assert_eq!(generator.as_mut().resume(()), GeneratorState::Yielded(1));
      std::thread::spawn(move || {
          assert_eq!(
              generator.into_iter().collect::<Vec<_>>(),
              Vec::from_iter(2..8),
          )
      })
      .join()
      .unwrap();
    }

    /// Ensure that the [`Send`] trait is implemented when the [`Generator`]s
    /// yields a value that is not [`Send`], as long as the value is not held
    /// across [`yield_`].
    #[test]
    fn non_send_yield() {
      #[generator(yield(Cell<u8>))]
      fn range(start: u8, end: u8) {
          let mut current: u8 = start;
          while current < end {
              yield_!(Cell::new(current));
              current += 1;
          }
      }
      mk_gen!(let mut generator = box range(1, 8));
      match generator.as_mut().resume(()) {
          GeneratorState::Yielded(x) => {
              assert_eq!(x.get(), 1);
          },
          _ => {
              panic!("expected yield");
          },
      };
      std::thread::spawn(move || {
          assert_eq!(
              generator
                  .into_iter()
                  .map(|cell| cell.get())
                  .collect::<Vec<_>>(),
              Vec::from_iter(2..8),
          )
      })
      .join()
      .unwrap();
    }

    /// Ensure that the [`Send`] trait is implemented when the [`Generator`]s
    /// has a resume argument that is not [`Send`], as long as the resume
    /// argument is not held across [`yield_`].
    #[test]
    fn non_send_resume() {
      #[generator(yield(u8), resume(Cell<u8>))]
      fn range(start: u8, end: u8) {
          let mut current: u8 = start;
          while current < end {
              let step = yield_!(current);
              current += step.get();
          }
      }
      mk_gen!(let mut generator = box range(1, 8));
      match generator.as_mut().resume(Cell::new(1)) {
          GeneratorState::Yielded(x) => {
              assert_eq!(x, 1);
          },
          _ => {
              panic!("expected yield");
          },
      };
      std::thread::spawn(move || {
          let mut got = Vec::new();
          loop {
              match generator.as_mut().resume(Cell::new(1)) {
                  GeneratorState::Yielded(x) => {
                      got.push(x);
                  },
                  GeneratorState::Returned(()) => {
                      break;
                  },
              }
          }
          assert_eq!(got, Vec::from_iter(2..8),)
      })
      .join()
      .unwrap();
    }
}

macro_rules! assert_it_eq {(
    $left:expr, $right:expr $(, $($msg:expr $(,)?)?)?
) => (
    assert_eq!(
        $left.into_iter().collect::<Vec<_>>(),
        $right,
        $($($msg ,)?)?
    )
)}
use assert_it_eq;

macro_rules! with_dollar {( $($rules:tt)* ) => (
    macro_rules! __emit__ { $($rules)* }
    __emit__! { $ }
)}
use with_dollar;

macro_rules! make_yield {(
    $yield_slot:expr $(,)?
) => (
    with_dollar! {( $_:tt ) => (
        macro_rules! yield_ {( $value:expr $_(,)? ) => (
            $yield_slot.__put($value).await
        )}
    )}
)}
use make_yield;
