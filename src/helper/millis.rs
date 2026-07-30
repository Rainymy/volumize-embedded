use avr_device::interrupt::{self, Mutex};
use core::cell::Cell;

static MILLIS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

// Called from the Timer0 overflow interrupt
pub fn tick() {
    interrupt::free(|cs| {
        let brw = MILLIS.borrow(cs);
        let millis = brw.get();
        brw.set(millis.saturating_add(1));
    });
}

pub fn millis() -> u32 {
    interrupt::free(|cs| { MILLIS.borrow(cs).get() })
}