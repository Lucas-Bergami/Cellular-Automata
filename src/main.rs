use iced::{Application, Settings, Size};
mod controller;
mod model;
mod view;

use controller::app::CASimulator;
/// Função principal do programa
pub fn main() -> iced::Result {
    CASimulator::run(Settings {
        window: iced::window::Settings {
            size: Size::new(1024.0, 768.0),
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}
