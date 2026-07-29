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

const VALUE_MIN: i32 = 0;
const VALUE_MAX: i32 = 100;
const STEPS_PER_CLICK: i32 = 1;

// ================= Shared state (ISR <-> main) =================
use avr_device::interrupt::Mutex;
use core::cell::Cell;

static ROT_STATE: Mutex<Cell<u8>> = Mutex::new(Cell::new(R_START));
static COUNTER: Mutex<Cell<i32>> = Mutex::new(Cell::new(10));

use arduino_hal::hal::port::{PD2, PD3};
use arduino_hal::port::{
    mode::{Input, PullUp},
    Pin,
};

type DtPin = Pin<Input<PullUp>, PD3>;
type ClkPin = Pin<Input<PullUp>, PD2>;

static DT_PIN: Mutex<Cell<Option<DtPin>>> = Mutex::new(Cell::new(None));
static CLK_PIN: Mutex<Cell<Option<ClkPin>>> = Mutex::new(Cell::new(None));

pub fn init_rotary(dt_pin: DtPin, clk_pin: ClkPin) {
    avr_device::interrupt::free(|cs| {
        DT_PIN.borrow(cs).set(Some(dt_pin));
        CLK_PIN.borrow(cs).set(Some(clk_pin));
    });
}

pub fn read_rotation_value() -> i32 {
    avr_device::interrupt::free(|cs| COUNTER.borrow(cs).get())
}

pub fn update_encoder() {
    avr_device::interrupt::free(|cs| {
        let dt_pin = DT_PIN.borrow(cs).take().unwrap();
        let dt = dt_pin.is_high() as u8;
        DT_PIN.borrow(cs).set(Some(dt_pin));

        let clk_pin = CLK_PIN.borrow(cs).take().unwrap();
        let clk = clk_pin.is_high() as u8;
        CLK_PIN.borrow(cs).set(Some(clk_pin));

        let pin_state = (clk << 1) | dt;

        let state_cell = ROT_STATE.borrow(cs);
        let old_state = state_cell.get() & 0xF;
        let new_state = TRANSITION_TABLE[old_state as usize][pin_state as usize];
        state_cell.set(new_state);

        let result = new_state & 0x30;
        let counter_cell = COUNTER.borrow(cs);
        let mut counter = counter_cell.get();

        if result == DIR_CW && counter < VALUE_MAX {
            counter += STEPS_PER_CLICK;
        } else if result == DIR_CCW && counter > VALUE_MIN {
            counter -= STEPS_PER_CLICK;
        }

        counter_cell.set(counter);
    });
}
