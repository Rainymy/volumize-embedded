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

const STEPS_PER_CLICK: RotaryType = RotaryType(1.0);

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

// ================= Shared state (ISR <-> main) =================
use core::cell::{Cell, RefCell};
use critical_section::{CriticalSection, Mutex};
use esp_hal::gpio::{AnyPin, Input};

static ROT_STATE: Mutex<Cell<u8>> = Mutex::new(Cell::new(R_START));
static COUNTER: Mutex<Cell<RotaryType>> = Mutex::new(Cell::new(RotaryType(10.0)));

type SetPin<T> = Mutex<RefCell<Option<T>>>;
static DT_PIN: SetPin<Input<'static>> = Mutex::new(RefCell::new(None));
static CLK_PIN: SetPin<Input<'static>> = Mutex::new(RefCell::new(None));

pub fn init_rotary(dt_pin: AnyPin<'static>, clk_pin: AnyPin<'static>) {
    critical_section::with(|cs| {
        use esp_hal::gpio::{Event, InputConfig, Pull};

        let input_config = InputConfig::default().with_pull(Pull::Up);
        let mut dt_pin = Input::new(dt_pin, input_config);
        let mut clk_pin = Input::new(clk_pin, input_config);

        dt_pin.listen(Event::AnyEdge);
        clk_pin.listen(Event::AnyEdge);

        DT_PIN.borrow(cs).replace(Some(dt_pin));
        CLK_PIN.borrow(cs).replace(Some(clk_pin));
    });
}

/// Reading value is capped between [VALUE_MIN] - [VALUE_MAX]
pub fn read_rotation_value() -> f32 {
    critical_section::with(|cs| COUNTER.borrow(cs).get().value())
}

/// Reads a pin's current level as 0/1 and clears its pending interrupt.
/// Must be called from inside a `critical_section`.
fn read_and_clear(cs: CriticalSection, pin: &SetPin<Input<'static>>) -> u8 {
    let mut guard = pin.borrow_ref_mut(cs);

    match guard.as_mut() {
        Some(pin) => {
            let level = pin.is_high() as u8;
            pin.clear_interrupt();
            level
        }
        None => {
            defmt::warn!("pin is not set; call `{}`", stringify!(init_rotary));
            0
        }
    }
}

/**
 * Honestly I have no idea why this works or what it does.
 * Just trusting that it has no edge cases.
 */
pub fn update_encoder() {
    critical_section::with(|cs| {
        let dt = read_and_clear(cs, &DT_PIN);
        let clk = read_and_clear(cs, &CLK_PIN);
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
    });
}
