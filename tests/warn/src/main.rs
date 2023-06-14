#![no_std]
#![no_main]

use voladdress::{Safe, VolAddress};

/// This address is used to communicate the current execution status directly with the test runner.
const STATUS_REGISTER: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0x0203FFFF) };

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub fn __sync_synchronize() {}

#[no_mangle]
pub fn main() {
    mgba_log::init().expect("unable to initialize");
    log::warn!("Hello, world!");

    STATUS_REGISTER.write(3);

    loop {}
}
