use crate::controller::{app::CASimulator, message::Message};
use iced::{
    widget::{button, column, row, text, text_input, Canvas, PickList, Scrollable, Slider},
    Alignment, Element, Length,
};
impl CASimulator {
    pub fn view_simulation_tab(&self) -> Element<Message> {
        let controls = column![
            text("Simulation Controls").size(20),
            row![
                text("Grid Width:"),
                text_input("e.g., 50", &self.grid_width_input)
                    .on_input(Message::GridWidthChanged)
                    .padding(3)
                    .width(Length::Fixed(60.0)),
                text("Grid Height:"),
                text_input("e.g., 40", &self.grid_height_input)
                    .on_input(Message::GridHeightChanged)
                    .padding(3)
                    .width(Length::Fixed(60.0)),
                button("Apply Size")
                    .on_press(Message::ApplyGridSize)
                    .padding(5),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
            row![
                button(if self.is_simulating { "Pause" } else { "Start" })
                    .on_press(Message::ToggleSimulation)
                    .padding(5),
                button("Next Step").on_press(Message::NextStep).padding(5),
                button("Reset Grid").on_press(Message::ResetGrid).padding(5),
            ]
            .spacing(10),
            row![
                text("Speed (Fast -> Slow):"),
                Slider::new(
                    0.0..=100.0,
                    100.0 - ((self.simulation_speed_ms.saturating_sub(10)) as f32 / 9.9),
                    Message::SimulationSpeedChanged
                )
                .width(Length::Fixed(200.0)),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
            text("Click on grid to paint state:"),
            PickList::new(
                self.states.clone(),
                self.states
                    .iter()
                    .find(|s| s.id == self.selected_paint_state_id)
                    .cloned(),
                Message::PaintStateSelected
            )
            .placeholder("Select Paint State"),
        ]
        .spacing(15)
        .width(Length::Fill); // faz o painel de controles expandir horizontalmente

        let canvas = Canvas::new(self)
            .width(Length::Fixed(600.0))
            .height(Length::Fixed(600.0));

        Scrollable::new(
            column![controls, canvas]
                .spacing(20)
                .align_items(Alignment::Start) // alinha à esquerda para o scroll “grudar” no lado direito
                .width(Length::Fill), // ocupa todo o espaço disponível
        )
        .width(Length::Fill) // garante que o Scrollable também preencha
        .into()
    }
}
