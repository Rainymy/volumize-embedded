use alloc::vec::Vec;
use core::cell::{Cell, RefCell};

use critical_section::{CriticalSection, Mutex};
use esp_hal::gpio::{AnyPin, Event, Input, InputConfig, Pull};

// ================= State machine constants =================
const R_START: u8 = 0x0;
const R_CW_FINAL: u8 = 0x1;
const R_CW_BEGIN: u8 = 0x2;
const R_CW_NEXT: u8 = 0x3;
const R_CCW_BEGIN: u8 = 0x4;
const R_CCW_FINAL: u8 = 0x5;
const R_CCW_NEXT: u8 = 0x6;

const DIR_CW: u8 = 0x10;
const DIR_CCW: u8 = 0x20;

const TRANSITION_TABLE: [[u8; 4]; 7] = [
    // R_START
    [R_START, R_CW_BEGIN, R_CCW_BEGIN, R_START],
    // R_CW_FINAL
    [R_CW_NEXT, R_START, R_CW_FINAL, R_START | DIR_CW],
    // R_CW_BEGIN
    [R_CW_NEXT, R_CW_BEGIN, R_START, R_START],
    // R_CW_NEXT
    [R_CW_NEXT, R_CW_BEGIN, R_CW_FINAL, R_START],
    // R_CCW_BEGIN
    [R_CCW_NEXT, R_START, R_CCW_BEGIN, R_START],
    // R_CCW_FINAL
    [R_CCW_NEXT, R_CCW_FINAL, R_START, R_START | DIR_CCW],
    // R_CCW_NEXT
    [R_CCW_NEXT, R_CCW_FINAL, R_CCW_BEGIN, R_START],
];

#[derive(Debug, Clone, Copy)]
struct RotaryType(f32);
impl RotaryType {
    pub const VALUE_MIN: f32 = 0.0;
    pub const VALUE_MAX: f32 = 100.0;

    fn limit_bound(value: f32) -> f32 {
        value.clamp(Self::VALUE_MIN, Self::VALUE_MAX)
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn saturating_add(&self, other: Self) -> Self {
        RotaryType(Self::limit_bound(self.0 + other.0))
    }

    pub fn saturating_sub(&self, other: Self) -> Self {
        RotaryType(Self::limit_bound(self.0 - other.0))
    }
}

type PinRef<T> = Mutex<RefCell<Option<T>>>;
type Queue<T> = Mutex<RefCell<Vec<T>>>;

pub static BUTTON: PinRef<Input<'static>> = Mutex::new(RefCell::new(None));
pub static EDGE_QUEUE: Queue<(bool, u64)> = Mutex::new(RefCell::new(Vec::new()));
static LAST_EDGE_MS: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));
static LAST_ACCEPTED_LEVEL: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

const DEBOUNCE_MS: u64 = 30;

// Rotary state
static ROT_STATE: Mutex<Cell<u8>> = Mutex::new(Cell::new(R_START));
static COUNTER: Mutex<Cell<RotaryType>> = Mutex::new(Cell::new(RotaryType(10.0)));

const STEPS_PER_CLICK: RotaryType = RotaryType(1.0);
pub static DT_PIN: PinRef<Input<'static>> = Mutex::new(RefCell::new(None));
pub static CLK_PIN: PinRef<Input<'static>> = Mutex::new(RefCell::new(None));

// ================== Init interrupt pins  ===================
pub fn init_button_interrupt(pin: AnyPin<'static>) {
    critical_section::with(|cs| {
        let config = InputConfig::default().with_pull(Pull::Up);
        let mut button = Input::new(pin, config);
        button.listen(Event::AnyEdge);

        BUTTON.borrow(cs).replace(Some(button));
    });
}

pub fn init_rotary_interrupt(dt_pin: AnyPin<'static>, clk_pin: AnyPin<'static>) {
    critical_section::with(|cs| {
        let input_config = InputConfig::default().with_pull(Pull::Up);
        let mut dt_pin = Input::new(dt_pin, input_config);
        let mut clk_pin = Input::new(clk_pin, input_config);

        dt_pin.listen(Event::AnyEdge);
        clk_pin.listen(Event::AnyEdge);

        DT_PIN.borrow(cs).replace(Some(dt_pin));
        CLK_PIN.borrow(cs).replace(Some(clk_pin));
    });
}

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
        if is_interrupted(cs, &DT_PIN) || is_interrupted(cs, &CLK_PIN) {
            let dt = read_high_and_clear(cs, &DT_PIN) as u8;
            let clk = read_high_and_clear(cs, &CLK_PIN) as u8;
            let pin_state = (clk << 1) | dt;

            let state_cell = ROT_STATE.borrow(cs);
            let old_state = state_cell.get() & 0xF;
            let new_state = TRANSITION_TABLE[old_state as usize][pin_state as usize];
            state_cell.set(new_state);

            let result = new_state & 0x30;
            let counter_cell = COUNTER.borrow(cs);

            counter_cell.set(match result {
                DIR_CW => counter_cell.get().saturating_add(STEPS_PER_CLICK),
                DIR_CCW => counter_cell.get().saturating_sub(STEPS_PER_CLICK),
                _ => counter_cell.get(),
            });
        }

        if is_interrupted(cs, &BUTTON) {
            use esp_hal::time::Instant;

            let level = !read_high_and_clear(cs, &BUTTON);
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
    });
}

// ================= Shared state (ISR <-> main) =================
/// Reading value is capped between [VALUE_MIN] - [VALUE_MAX]
pub fn read_rotation_value() -> f32 {
    critical_section::with(|cs| COUNTER.borrow(cs).get().value())
}

pub fn with_edge_queue<F>(mut f: F)
where
    F: FnMut(bool, u64),
{
    critical_section::with(|cs| {
        let mut queue = EDGE_QUEUE.borrow_ref_mut(cs);
        for (is_down, timestamp) in queue.iter() {
            f(*is_down, *timestamp);
        }
        queue.clear();
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
