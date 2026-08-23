use alloc::vec::Vec;
use core::cell::RefCell;

use critical_section::{CriticalSection, Mutex};
use esp_hal::gpio::{AnyPin, Event, Input, InputConfig, Pull};

mod button;
mod rotary;

pub use button::*;
pub use rotary::*;

type PinRef<T> = Mutex<RefCell<Option<T>>>;
type Queue<T> = Mutex<RefCell<Vec<T>>>;

// ================== Init interrupt pins  ===================
pub fn enable_gpio_interrupts() {
    use esp_hal::interrupt;
    use esp_hal::interrupt::{InterruptHandler, Priority};
    use esp_hal::peripherals::Interrupt;

    interrupt::bind_handler(
        Interrupt::GPIO,
        InterruptHandler::new(gpio_interrupt_handler, Priority::min()),
    );
}

pub extern "C" fn gpio_interrupt_handler() {
    critical_section::with(|cs| {
        rotary::interrupt_handler(cs);
        button::interrupt_handler(cs);
    });
}

// ==================== Helper functions =====================
fn is_interrupted(cs: CriticalSection, pin: &PinRef<Input<'static>>) -> bool {
    let pin_ref = pin.borrow_ref(cs);
    match pin_ref.as_ref() {
        Some(pin) => pin.is_interrupt_set(),
        None => false,
    }
}

/// Reads a pin's current level as 0/1 and clears its pending interrupt.
/// Must be called from inside a `critical_section`.
fn read_high_and_clear(cs: CriticalSection, pin: &PinRef<Input<'static>>) -> bool {
    let mut guard = pin.borrow_ref_mut(cs);

    match guard.as_mut() {
        Some(pin) => {
            let level = pin.is_high();
            pin.clear_interrupt();
            level
        }
        None => {
            defmt::warn!("pin is not set");
            false
        }
    }
}

fn replace_pin_default(
    cs: CriticalSection,
    pin_ref: &PinRef<Input<'static>>,
    pin: AnyPin<'static>,
) {
    let config = InputConfig::default().with_pull(Pull::Up);
    let mut pin = Input::new(pin, config);

    pin.listen(Event::AnyEdge);

    pin_ref.borrow(cs).replace(Some(pin));
}
