use_prelude!();

pub(in crate)
fn poll_fn<'F, F : 'F, T> (f: F)
  -> impl 'F + Unpin + Future<Output = T>
where
    F : FnMut(&mut Context<'_>) -> Poll<T>,
{
    struct PollFn<F> {
        f: F,
    }

    /// No pinning projection.
    impl<F> Unpin for PollFn<F> {}

    impl<F> ::core::fmt::Debug for PollFn<F> {
        fn fmt (self: &'_ PollFn<F>, f: &'_ mut ::core::fmt::Formatter<'_>)
          -> ::core::fmt::Result
        {
            f   .debug_struct("PollFn")
                .finish()
        }
    }

    impl<T, F> Future for PollFn<F>
    where
        F : FnMut(&'_ mut Context<'_>) -> Poll<T>,
    {
        type Output = T;

        fn poll (
            mut self: Pin<&'_ mut PollFn<F>>,
            cx: &'_ mut Context<'_>,
        ) -> Poll<T>
        {
            (&mut self.f)(cx)
        }
    }

    PollFn { f }
}
