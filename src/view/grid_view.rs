use crate::messages::Message;
use crate::CASimulator;
use iced::widget::canvas;
use iced::widget::canvas::{Geometry, Path, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme, Vector};

impl canvas::Program<Message> for CASimulator {
    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let grid_geometry = self.grid_cache.draw(renderer, bounds.size(), |frame| {
            if self.grid.width == 0 || self.grid.height == 0 || self.states.is_empty() {
                let placeholder_text = canvas::Text {
                    content: "Grid not initialized or no states.".to_string(),
                    position: frame.center(),
                    color: Color::WHITE,
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    ..Default::default()
                };
                frame.fill_text(placeholder_text);
                return;
            }

            frame.with_save(|frame| {
                let zoom = self.zoom.get().max(0.1);
                let offset = self.offset.get();

                frame.translate(Vector::new(offset.x, offset.y));
                frame.scale(zoom);

                let cell_width = frame.width() / self.grid.width as f32;
                let cell_height = frame.height() / self.grid.height as f32;

                for r in 0..self.grid.height {
                    for c in 0..self.grid.width {
                        let state_id = self.grid.cells[r][c];
                        let cell_color = self
                            .states
                            .iter()
                            .find(|s| s.id == state_id)
                            .map_or(Color::new(1.0, 0.0, 0.0, 1.0), |s| s.color);

                        let top_left = Point::new(c as f32 * cell_width, r as f32 * cell_height);
                        let size = Size::new(cell_width, cell_height);

                        frame.fill_rectangle(top_left, size, cell_color);
                    }
                }

                let min_cell_pixels = 1.5;
                let draw_horizontal = cell_height * zoom >= min_cell_pixels;
                let draw_vertical = cell_width * zoom >= min_cell_pixels;

                if draw_horizontal || draw_vertical {
                    let stroke_width = (1.5 / zoom).clamp(0.5, 3.0);
                    let stroke_color = Color::from_rgb(0.2, 0.2, 0.2);

                    // Linhas horizontais
                    if draw_horizontal {
                        for r in 0..=self.grid.height {
                            let y = r as f32 * cell_height;
                            let path = Path::line(Point::new(0.0, y), Point::new(frame.width(), y));
                            frame.stroke(
                                &path,
                                Stroke::default()
                                    .with_width(stroke_width)
                                    .with_color(stroke_color),
                            );
                        }
                    }

                    if draw_vertical {
                        for c in 0..=self.grid.width {
                            let x = c as f32 * cell_width;
                            let path =
                                Path::line(Point::new(x, 0.0), Point::new(x, frame.height()));
                            frame.stroke(
                                &path,
                                Stroke::default()
                                    .with_width(stroke_width)
                                    .with_color(stroke_color),
                            );
                        }
                    }
                }
            });
        });

        vec![grid_geometry]
    }

    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        if let canvas::Event::Mouse(mouse_event) = event {
            match mouse_event {
                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                    self.mouse_pressed.set(true);
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                    self.mouse_pressed.set(false);
                    *self.last_painted_cell.borrow_mut() = None;
                }

                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right) => {
                    self.right_mouse_pressed.set(true);
                }
                iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right) => {
                    self.right_mouse_pressed.set(false);
                    *self.last_mouse_pos.borrow_mut() = None;
                }

                iced::mouse::Event::WheelScrolled { delta } => {
                    if let Some(position) = cursor.position_in(bounds) {
                        let zoom_factor = match delta {
                            iced::mouse::ScrollDelta::Lines { y, .. } => y,
                            iced::mouse::ScrollDelta::Pixels { y, .. } => y / 100.0,
                        };

                        let old_zoom = self.zoom.get();
                        let mut new_zoom = old_zoom + zoom_factor * 0.1;
                        new_zoom = new_zoom.clamp(0.1, 10.0);
                        self.zoom.set(new_zoom);

                        let offset = self.offset.get();
                        let grid_x = (position.x - offset.x) / old_zoom;
                        let grid_y = (position.y - offset.y) / old_zoom;
                        let new_offset = iced::Point::new(
                            position.x - grid_x * new_zoom,
                            position.y - grid_y * new_zoom,
                        );
                        self.offset.set(new_offset);

                        self.grid_cache.clear();
                        return (canvas::event::Status::Captured, None);
                    }
                }

                iced::mouse::Event::CursorMoved { position } => {
                    if self.right_mouse_pressed.get() {
                        self.grid_cache.clear();
                        let mut offset = self.offset.get();
                        if let Some(last_pos) = *self.last_mouse_pos.borrow() {
                            let dx = position.x - last_pos.x;
                            let dy = position.y - last_pos.y;
                            offset.x += dx;
                            offset.y += dy;
                            self.offset.set(offset);
                        }
                        *self.last_mouse_pos.borrow_mut() = Some(position);
                        return (canvas::event::Status::Captured, None);
                    } else {
                        *self.last_mouse_pos.borrow_mut() = Some(position);
                    }
                }

                _ => {}
            }
        }

        if self.mouse_pressed.get()
            && let Some(position) = cursor.position_in(bounds)
        {
            let offset = self.offset.get();
            let adjusted_x = (position.x - offset.x) / self.zoom.get();
            let adjusted_y = (position.y - offset.y) / self.zoom.get();

            let cell_width = bounds.width / self.grid.width as f32;
            let cell_height = bounds.height / self.grid.height as f32;

            let col = (adjusted_x / cell_width) as usize;
            let row = (adjusted_y / cell_height) as usize;

            if row < self.grid.height && col < self.grid.width {
                let mut last = self.last_painted_cell.borrow_mut();
                if last.is_none() || last.unwrap() != (row, col) {
                    *last = Some((row, col));
                    return (
                        canvas::event::Status::Captured,
                        Some(Message::PaintCell(row, col, self.selected_paint_state_id)),
                    );
                }
            }
        }

        self.grid_cache.clear();
        (canvas::event::Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        if cursor.is_over(bounds) {
            iced::mouse::Interaction::Crosshair
        } else {
            iced::mouse::Interaction::default()
        }
    }
}
