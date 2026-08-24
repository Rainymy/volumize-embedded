pub mod adjust_volume;
pub mod application_menu;
pub mod settings;
pub mod system_menu;
pub mod ui_state;

use adjust_volume::{VolumeAdjustState, handle_volume_adjust};
use application_menu::{ApplicationMenuState, handle_main_menu};
use settings::{SettingsState, handle_settings};
use system_menu::{SystemMenuState, handle_system_menu};

pub use ui_state::UIState;

use crate::InputEvent;

#[allow(dead_code)]
pub enum Transition {
    Stay,         // event handled, no navigation change
    Push(Screen), // enter a new screen (e.g. select a menu item)
    Pop,          // go back to the previous screen
    Ignored,      // event didn't apply here (optional, same as Stay)
}

// #[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Screen {
    ApplicationList(ApplicationMenuState),
    SystemMenu(SystemMenuState),
    VolumeAdjust(VolumeAdjustState),
    Settings(SettingsState),
}

impl Default for Screen {
    fn default() -> Self {
        Self::ApplicationList(ApplicationMenuState::new())
    }
}

pub fn handle_event(ui_state: &mut UIState, event: InputEvent) {
    let transition = match ui_state.current_mut() {
        Screen::ApplicationList(state) => handle_main_menu(state, event),
        Screen::VolumeAdjust(state) => handle_volume_adjust(state, event),
        Screen::Settings(state) => handle_settings(state, event),
        Screen::SystemMenu(state) => handle_system_menu(state, event),
    };

    match transition {
        Transition::Stay | Transition::Ignored => {}
        Transition::Push(screen) => ui_state.push(screen),
        Transition::Pop => ui_state.pop(),
    }
}
