use core::cell::{Cell, RefCell};

use critical_section::Mutex;
use esp_hal::gpio::{AnyPin, Input};

#[derive(Debug, Clone, Copy)]
struct RotaryType(pub u8);
impl RotaryType {
    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn wrapping_add(&self, other: u8) -> Self {
        RotaryType(self.0.wrapping_add(other))
    }

    pub fn wrapping_sub(&self, other: u8) -> Self {
        RotaryType(self.0.wrapping_sub(other))
    }
}

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

use super::PinRef;
use super::is_interrupted;

// Rotary state
static ROT_STATE: Mutex<Cell<u8>> = Mutex::new(Cell::new(R_START));
static COUNTER: Mutex<Cell<RotaryType>> = Mutex::new(Cell::new(RotaryType(0)));

static DT_PIN: PinRef<Input<'static>> = Mutex::new(RefCell::new(None));
static CLK_PIN: PinRef<Input<'static>> = Mutex::new(RefCell::new(None));

pub fn init_rotary_interrupt(dt_pin: AnyPin<'static>, clk_pin: AnyPin<'static>) {
    critical_section::with(|cs| {
        super::replace_pin_default(cs, &DT_PIN, dt_pin);
        super::replace_pin_default(cs, &CLK_PIN, clk_pin);
    });
}

pub fn interrupt_handler(cs: critical_section::CriticalSection) {
    if is_interrupted(cs, &DT_PIN) || is_interrupted(cs, &CLK_PIN) {
        let dt = super::read_high_and_clear(cs, &DT_PIN) as u8;
        let clk = super::read_high_and_clear(cs, &CLK_PIN) as u8;
        let pin_state = (clk << 1) | dt;

        let state_cell = ROT_STATE.borrow(cs);
        let old_state = state_cell.get() & 0xF;
        let new_state = TRANSITION_TABLE[old_state as usize][pin_state as usize];
        state_cell.set(new_state);

        let result = new_state & 0x30;
        let counter_cell = COUNTER.borrow(cs);

        counter_cell.set(match result {
            DIR_CW => counter_cell.get().wrapping_add(1),
            DIR_CCW => counter_cell.get().wrapping_sub(1),
            _ => counter_cell.get(),
        });
    }
}

/// Reading value is capped between [VALUE_MIN] - [VALUE_MAX]
pub fn read_rotation_value() -> u8 {
    critical_section::with(|cs| COUNTER.borrow(cs).get().value())
}
