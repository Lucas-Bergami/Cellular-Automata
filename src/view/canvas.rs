use crate::controller::app::CASimulator;
use crate::controller::message::Message;
use crate::view::canvas::canvas::Geometry;
use iced::widget::canvas::{self};
use iced::widget::Renderer;
use iced::{Rectangle, Theme};

// Implement canvas::Program for CASimulator to draw the grid
impl canvas::Program<Message> for CASimulator {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let grid_geometry = self.grid_cache.draw(renderer, bounds.size(), |frame| {
            self.draw_grid(frame);
        });
        vec![grid_geometry]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if let canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) =
            event
        {
            if let Some(position) = cursor.position_in(bounds) {
                if let Some(msg) = self.handle_click(position, bounds) {
                    return (canvas::event::Status::Captured, Some(msg));
                }
            }
        }

        (canvas::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if cursor.is_over(bounds) {
            iced::mouse::Interaction::Crosshair // Or Pointer, etc.
        } else {
            iced::mouse::Interaction::default()
        }
    }
}
