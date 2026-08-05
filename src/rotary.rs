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

type RotaryType = i8;
const VALUE_MIN: RotaryType = 0;
const VALUE_MAX: RotaryType = 100;
const STEPS_PER_CLICK: RotaryType = 1;

// ================= Shared state (ISR <-> main) =================
use core::cell::Cell;
use critical_section::Mutex;

static ROT_STATE: Mutex<Cell<u8>> = Mutex::new(Cell::new(R_START));
static COUNTER: Mutex<Cell<RotaryType>> = Mutex::new(Cell::new(10));

use esp_hal::gpio::{AnyPin, Input};

type DtPin = AnyPin<'static>;
type ClkPin = AnyPin<'static>;
type SDtPin = Input<'static>;
type SClkPin = Input<'static>;

static DT_PIN: Mutex<Cell<Option<SDtPin>>> = Mutex::new(Cell::new(None));
static CLK_PIN: Mutex<Cell<Option<SClkPin>>> = Mutex::new(Cell::new(None));

pub fn init_rotary(dt_pin: DtPin, clk_pin: ClkPin) {
    critical_section::with(|cs| {
        use esp_hal::gpio::{Event, InputConfig, Pull};

        let input_config = InputConfig::default().with_pull(Pull::Up);
        let mut dt_pin = Input::new(dt_pin, input_config);
        let mut clk_pin = Input::new(clk_pin, input_config);

        dt_pin.listen(Event::AnyEdge);
        clk_pin.listen(Event::AnyEdge);

        DT_PIN.borrow(cs).set(Some(dt_pin));
        CLK_PIN.borrow(cs).set(Some(clk_pin));
    });
}

pub fn read_rotation_value() -> RotaryType {
    critical_section::with(|cs| COUNTER.borrow(cs).get())
}

/**
 * Honestly I have no idea why this works or what it does.
 * Just trusting that it has no edge cases.
 */
pub fn update_encoder() {
    critical_section::with(|cs| {
        // TODO: Needs better type or way. Instead of taking and setting.
        let dt_pin = DT_PIN
            .borrow(cs)
            .take()
            .expect("DT_PIN is not set; call `init_rotary`");
        let dt = dt_pin.is_high() as u8;
        DT_PIN.borrow(cs).set(Some(dt_pin));

        let clk_pin = CLK_PIN
            .borrow(cs)
            .take()
            .expect("CLK_PIN is not set; call `init_rotary`");
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
