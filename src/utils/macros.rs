#![allow(unused_imports)]

macro_rules! create_context {(
    $cx:ident
) => (
    let waker: ::core::task::Waker = $crate::waker::create();
    let mut $cx = ::core::task::Context::from_waker(&waker);
)} pub(in crate) use create_context;

macro_rules! abort_with_msg {( $($msg:tt)* ) => ({
    let () =
        $crate::__::unwind_safe::with_state(())
            .try_eval(|()| $crate::__::core::panic!($($msg)*))
            .finally(|()| $crate::__::core::panic!())
    ;
    loop {}
})} pub(in crate) use abort_with_msg;

#[allow(unused_macros)]
macro_rules! emit {( $($item:item)* ) => (
    $($item)*
)} pub(in crate) use emit;

#[allow(unused_macros)]
macro_rules! export_hidden_macros {(
    $(
        macro_rules ! $macro_name:ident $def:tt
    )*
) => (
    $(
        #[doc(hidden)] /** Not part of the public API */ #[macro_export]
        macro_rules! $macro_name $def
        pub use $macro_name;
    )*
)} pub(in crate) use export_hidden_macros;
