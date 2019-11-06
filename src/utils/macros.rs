macro_rules! use_prelude {() => (
    use crate::utils::prelude::*;
)}

macro_rules! create_context {(
    $cx:ident
) => (
    let waker: ::core::task::Waker = $crate::waker::create();
    let mut $cx = ::core::task::Context::from_waker(&waker);
)}

// macro_rules! pin_mut {(
//     $($ident:ident),*
//     $(,)?
// ) => (
//     let ($(ref mut $ident ,)*) = ($($ident ,)*);
//     $(
//         let $ident = unsafe {
//             ::core::pin::Pin::new_unchecked($ident)
//         };
//     )*
// )}
