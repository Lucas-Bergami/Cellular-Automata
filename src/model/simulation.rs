use crate::controller::app::CASimulator;
use crate::controller::message::Message;
use crate::model::simulation::canvas::{Path, Stroke, Text};
use iced::widget::canvas;
use iced::{Color, Point, Rectangle, Size};

impl CASimulator {
    // fn view_model_image_tab(&self) -> Element<Message> {
    //     // This tab is for the "image of the created model".
    //     // For now, it could just be a textual summary or a placeholder.
    //     // A true graphical representation of rules/states (like a graph) is complex.
    //     container(
    //         text("Model Image / Summary (Placeholder)")
    //             .horizontal_alignment(iced::alignment::Horizontal::Center)
    //             .vertical_alignment(iced::alignment::Vertical::Center)
    //     )
    //     .width(Length::Fill)
    //     .height(Length::Fill)
    //     .center_x()
    //     .center_y()
    //     .into()
    // }

    pub fn step_simulation_logic(&mut self) {
        if self.states.is_empty() {
            return;
        } // No states, nothing to do

        let mut next_grid_cells = self.grid.cells.clone();
        let current_grid = &self.grid; // Immutable borrow for reading

        for r in 0..current_grid.height {
            for c in 0..current_grid.width {
                let current_cell_state_id = current_grid.cells[r][c];
                let mut new_state_id = current_cell_state_id; // Default to no change

                for rule in &self.rules {
                    if rule.current_state_id == current_cell_state_id {
                        let neighbor_count =
                            current_grid.count_neighbors(r, c, rule.neighbor_state_id_to_count);
                        if rule
                            .operator
                            .evaluate(neighbor_count, rule.neighbor_count_threshold)
                        {
                            new_state_id = rule.next_state_id;
                            break; // First matching rule applies
                        }
                    }
                }
                next_grid_cells[r][c] = new_state_id;
            }
        }
        self.grid.cells = next_grid_cells;
        self.grid_cache.clear(); // Crucial: Invalidate cache to force redraw
    }

    pub fn draw_grid(&self, frame: &mut canvas::Frame) {
        if self.grid.width == 0 || self.grid.height == 0 || self.states.is_empty() {
            let placeholder_text = Text {
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

        let cell_width = frame.width() / self.grid.width as f32;
        let cell_height = frame.height() / self.grid.height as f32;

        for r in 0..self.grid.height {
            for c in 0..self.grid.width {
                let state_id = self.grid.cells[r][c];
                let cell_color = self
                    .states
                    .iter()
                    .find(|s| s.id == state_id)
                    .map_or(Color::from_rgb(1.0, 0.0, 0.0), |s| s.color);

                let top_left = Point::new(c as f32 * cell_width, r as f32 * cell_height);
                let size = Size::new(cell_width, cell_height);
                frame.fill_rectangle(top_left, size, cell_color);

                if cell_width > 4.0 && cell_height > 4.0 {
                    let path = Path::rectangle(top_left, size);
                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_width(1.5)
                            .with_color(Color::from_rgb(0.2, 0.2, 0.2)),
                    );
                }
            }
        }
    }

    pub fn handle_click(&self, position: Point, bounds: Rectangle) -> Option<Message> {
        if self.grid.width == 0 || self.grid.height == 0 {
            return None;
        }

        let cell_width = bounds.width / self.grid.width as f32;
        let cell_height = bounds.height / self.grid.height as f32;

        let col = (position.x / cell_width) as usize;
        let row = (position.y / cell_height) as usize;

        if row < self.grid.height && col < self.grid.width {
            Some(Message::PaintCell(row, col, self.selected_paint_state_id))
        } else {
            None
        }
    }
}

// TESTAR
// pub fn step_simulation(
//     grid: &CAGrid,
//     rules: &[TransitionRule],
// ) -> CAGrid {
//     let mut next_grid_cells = grid.cells.clone();
//
//     for r in 0..grid.height {
//         for c in 0..grid.width {
//             let current_cell_state_id = grid.cells[r][c];
//             let mut new_state_id = current_cell_state_id;
//
//             for rule in rules {
//                 if rule.current_state_id == current_cell_state_id {
//                     let neighbor_count =
//                         grid.count_neighbors(r, c, rule.neighbor_state_id_to_count);
//                     if rule
//                         .operator
//                         .evaluate(neighbor_count, rule.neighbor_count_threshold)
//                     {
//                         new_state_id = rule.next_state_id;
//                         break;
//                     }
//                 }
//             }
//
//             next_grid_cells[r][c] = new_state_id;
//         }
//     }
//
//     CAGrid {
//         width: grid.width,
//         height: grid.height,
//         cells: next_grid_cells,
//     }
// }
