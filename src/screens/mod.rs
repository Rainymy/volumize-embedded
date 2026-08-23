#![allow(dead_code)]

mod adjust_volume;
mod main_menu;
mod settings;
mod ui_state;

use main_menu::MainMenuState;
pub use ui_state::UIState;

use crate::{
    InputEvent,
    screens::{
        adjust_volume::{VolumeAdjustState, handle_volume_adjust},
        main_menu::handle_main_menu,
        settings::{SettingsState, handle_settings},
    },
};

pub enum Transition {
    Stay,         // event handled, no navigation change
    Push(Screen), // enter a new screen (e.g. select a menu item)
    Pop,          // go back to the previous screen
    Ignored,      // event didn't apply here (optional, same as Stay)
}

#[derive(Debug, Clone)]
pub enum Screen {
    MainMenu(MainMenuState),
    VolumeAdjust(VolumeAdjustState),
    Settings(SettingsState),
}

impl Default for Screen {
    fn default() -> Self {
        Self::MainMenu(MainMenuState::default())
    }
}

pub fn handle_event(ui_state: &mut UIState, event: InputEvent) {
    let transition = match ui_state.current_mut() {
        Screen::MainMenu(state) => handle_main_menu(state, event),
        Screen::VolumeAdjust(state) => handle_volume_adjust(state, event),
        Screen::Settings(state) => handle_settings(state, event),
    };

    match transition {
        Transition::Stay | Transition::Ignored => {}
        Transition::Push(screen) => ui_state.push(screen),
        Transition::Pop => ui_state.pop(),
    }
}
