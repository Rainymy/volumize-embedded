use core::cell::RefCell;
use core::ops::Div;
use core::sync::atomic::{AtomicI32, Ordering};

use critical_section::Mutex;
use esp_hal::gpio::{AnyPin, Input, InputConfig, Pull};
use esp_hal::pcnt::{Pcnt, channel, unit};
use esp_hal::peripherals::PCNT;

const COUNTS_PER_DETENT: i16 = 4;
const LIMIT: i16 = 1000; // This value is arbitrary

type UnitRef = Mutex<RefCell<Option<unit::Unit<'static, 1>>>>;

static ROTARY_UNIT: UnitRef = Mutex::new(RefCell::new(None));
static POSITION_CARRY: AtomicI32 = AtomicI32::new(0);

pub fn init_rotary_interrupt(
    pcnt: PCNT<'static>,
    dt_pin: AnyPin<'static>,
    clk_pin: AnyPin<'static>,
) {
    let pin_config = InputConfig::default().with_pull(Pull::Up);
    let dt = Input::new(dt_pin, pin_config);
    let clk = Input::new(clk_pin, pin_config);
    let input_dt = dt.peripheral_input();
    let input_clk = clk.peripheral_input();

    let mut pcnt = Pcnt::new(pcnt);
    pcnt.set_interrupt_handler(interrupt_handler);

    let u0 = pcnt.unit1;
    u0.set_low_limit(Some(-LIMIT)).unwrap();
    u0.set_high_limit(Some(LIMIT)).unwrap();
    u0.set_filter(Some(800)).unwrap();
    u0.clear();

    let ch0 = &u0.channel0;
    ch0.set_ctrl_signal(input_dt.clone());
    ch0.set_edge_signal(input_clk.clone());
    ch0.set_ctrl_mode(channel::CtrlMode::Reverse, channel::CtrlMode::Keep);
    ch0.set_input_mode(channel::EdgeMode::Increment, channel::EdgeMode::Decrement);

    let ch1 = &u0.channel1;
    ch1.set_ctrl_signal(input_clk);
    ch1.set_edge_signal(input_dt);
    ch1.set_ctrl_mode(channel::CtrlMode::Reverse, channel::CtrlMode::Keep);
    ch1.set_input_mode(channel::EdgeMode::Decrement, channel::EdgeMode::Increment);

    u0.listen();
    u0.resume();

    critical_section::with(|cs| ROTARY_UNIT.borrow_ref_mut(cs).replace(u0));
}

#[esp_hal::handler]
pub fn interrupt_handler() {
    critical_section::with(|cs| {
        let mut unit_ref = ROTARY_UNIT.borrow_ref_mut(cs);
        if let Some(u0) = unit_ref.as_mut() {
            if u0.interrupt_is_set() {
                let events = u0.events();
                if events.high_limit {
                    POSITION_CARRY.fetch_add(LIMIT as i32, Ordering::SeqCst);
                } else if events.low_limit {
                    POSITION_CARRY.fetch_add(-(LIMIT as i32), Ordering::SeqCst);
                }
                u0.reset_interrupt();
            }
        }
    });
}

pub fn read_rotation_value() -> i16 {
    let raw = critical_section::with(|cs| {
        let unit_ref = ROTARY_UNIT.borrow_ref(cs);
        match unit_ref.as_ref() {
            Some(u0) => u0.counter.get(),
            None => 0,
        }
    });

    let position_carry = POSITION_CARRY.load(Ordering::SeqCst) as i16;

    raw.wrapping_add(position_carry).div(COUNTS_PER_DETENT)
}
