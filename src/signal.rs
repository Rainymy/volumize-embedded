use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

pub static READY: Signal<CriticalSectionRawMutex, ()> = Signal::new();
pub fn notify_ready() {
    READY.signal(());
}

pub async fn wait_for_ready() {
    READY.wait().await;
}
