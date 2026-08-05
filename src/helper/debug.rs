// use arduino_hal as hal;

// use hal::port::mode::Output;
// use hal::port::{Pin, PinOps};

// #[macro_export]
// macro_rules! log {
//     ($serial:expr, $($arg:tt)*) => {{
//         let _ = ufmt::uwrite!($serial, $($arg)*);
//     }};
// }

// /// Blinks the given output pin forever at the given delay (ms).
// /// Call this when something has failed and you want a visible, distinct signal.
// pub fn blink_forever<P: PinOps>(mut led: Pin<Output, P>, delay_ms: u16, blink_count: u16) -> ! {
//     let loop_count = blink_count * 2;
//     loop {
//         for _ in 0..loop_count {
//             led.toggle();
//             esp_hal::delay::Delay::new().delay_millis(delay_ms);
//             // hal::delay_ms(delay_ms as u32);
//         }

//         esp_hal::delay::Delay::new().delay_millis(2000);
//         // hal::delay_ms(2000);
//     }
// }
