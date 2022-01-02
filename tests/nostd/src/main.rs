#![no_std]
#![no_main]

#[allow(unused)]
use ::next_gen;

/// This function is called on panic.
#[panic_handler]
fn panic (_info: &'_ ::core::panic::PanicInfo)
  -> !
{
    loop {}
}

#[no_mangle] pub extern "C"
fn _start ()
  -> !
{
    loop {}
}
