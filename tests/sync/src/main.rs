#![no_std]
#![no_main]

extern crate gba;

use gba::{asm_runtime::RUST_IRQ_HANDLER, interrupts::IrqBits, mmio::{DISPSTAT, IE, IME}, video::DisplayStatus};
use voladdress::{Safe, VolAddress};

/// This address is used to communicate the current execution status directly with the test runner.
const STATUS_REGISTER: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0x0203FFFF) };

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[link_section = ".iwram"]
extern "C" fn irq_handler(_: IrqBits) {
  log::debug!("in irq");
}

#[no_mangle]
pub fn main() {
    mgba_log::init().expect("unable to initialize");

    RUST_IRQ_HANDLER.write(Some(irq_handler));
    DISPSTAT.write(DisplayStatus::new().with_irq_vblank(true));
    IE.write(IrqBits::VBLANK);
    IME.write(true);

    for _ in 0..1000 {
        log::info!("Hello, world!");
    }

    STATUS_REGISTER.write(3);

    loop {}
}
