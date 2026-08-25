use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use embassy_futures::block_on;

use critical_section::Mutex;
use esp_hal::gpio::{AnyPin, Input};

use super::{PinRef, Queue};

const DEBOUNCE_MS: u64 = 30;

static BUTTON: PinRef<Input<'static>> = Mutex::new(RefCell::new(None));
static EDGE_QUEUE: Queue<(bool, u64)> = Mutex::new(RefCell::new(Vec::new()));
static LAST_EDGE_MS: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));
static LAST_ACCEPTED_LEVEL: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

pub fn init_button_interrupt(pin: AnyPin<'static>) {
    critical_section::with(|cs| {
        super::replace_pin_default(cs, &BUTTON, pin);
    });
}

pub fn interrupt_handler(cs: critical_section::CriticalSection) {
    if super::is_interrupted(cs, &BUTTON) {
        use esp_hal::time::Instant;

        let level = !super::read_high_and_clear(cs, &BUTTON);
        let timestamp = Instant::now().duration_since_epoch().as_millis();

        let last_edge_ms = LAST_EDGE_MS.borrow(cs);
        let last_accepted_level = LAST_ACCEPTED_LEVEL.borrow(cs);

        let is_debounced = timestamp.saturating_sub(last_edge_ms.get()) > DEBOUNCE_MS;
        let is_level_changed = level != last_accepted_level.get();

        if is_debounced && is_level_changed {
            last_edge_ms.set(timestamp);
            last_accepted_level.set(level);

            let mut edge_queue = EDGE_QUEUE.borrow_ref_mut(cs);
            edge_queue.push((level, timestamp));
        }
    }
}

pub fn with_edge_queue<F>(mut f: F)
where
    F: AsyncFnMut(bool, u64),
{
    critical_section::with(|cs| {
        let mut queue = EDGE_QUEUE.borrow_ref_mut(cs);
        for (is_down, timestamp) in queue.iter() {
            block_on(f(*is_down, *timestamp));
        }
        queue.clear();
    });
}
