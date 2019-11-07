mod docs {
    macro_rules! test_include {(
        $($ident:ident),* $(,)?
    ) => (
        $(
            mod $ident {
                #[test] fn test () { main() }
                include! {
                    concat!(
                        "../doc_examples/",
                        stringify!($ident),
                        ".rs",
                    )
                }
            }

        )*
    )}
    test_include! {
        generator,
        generator_desugared,
    }
}
