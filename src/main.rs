mod app;
mod messages;
mod state;
mod view;

use crate::app::CASimulator;
use iced::{Application, Settings};

pub fn main() -> iced::Result {
    CASimulator::run(Settings {
        window: iced::window::Settings {
            size: iced::Size {
                width: 1024.0,
                height: 768.0,
            },
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}
