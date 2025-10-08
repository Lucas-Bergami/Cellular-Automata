use crate::state::CAState;
use serde::{Deserialize, Serialize};
use std::fmt;

// The 2D grid for simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CAGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<u8>>, // Stores state IDs
    pub neighborhood: Neighborhood,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Neighborhood {
    VonNeumann,
    Moore,
    ExtendedMoore,
}

impl fmt::Display for Neighborhood {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Neighborhood::VonNeumann => write!(f, "Von Neumann (4)"),
            Neighborhood::Moore => write!(f, "Moore (8)"),
            Neighborhood::ExtendedMoore => write!(f, "Extended Moore (16)"),
        }
    }
}

impl CAGrid {
    pub fn new(
        width: usize,
        height: usize,
        states: Vec<CAState>,
        neighborhood: Neighborhood,
    ) -> Self {
        use rand::Rng;

        let mut available_states: Vec<CAState> =
            states.into_iter().filter(|s| s.weight > 0).collect();

        if available_states.is_empty() {
            available_states.push(CAState {
                id: 0,
                name: "Default".to_string(),
                color: iced::Color::BLACK,
                weight: 1,
            });
        }

        let total_weight: u32 = available_states.iter().map(|s| s.weight as u32).sum();

        let mut rng = rand::rng();

        let cells = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| {
                        let mut roll = rng.random_range(0..total_weight);
                        for state in &available_states {
                            if roll < state.weight as u32 {
                                return state.id;
                            }
                            roll -= state.weight as u32;
                        }
                        available_states[0].id
                    })
                    .collect::<Vec<u8>>()
            })
            .collect::<Vec<Vec<u8>>>();

        CAGrid {
            width,
            height,
            cells,
            neighborhood,
        }
    }

    pub fn count_neighbors(&self, r: usize, c: usize, target_state_id: u8) -> u8 {
        let directions: &[(isize, isize)] = match self.neighborhood {
            Neighborhood::VonNeumann => &[(-1, 0), (1, 0), (0, -1), (0, 1)],
            Neighborhood::Moore => &[
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ],
            Neighborhood::ExtendedMoore => &[
                // normal Moore
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
                // Second layer
                (-2, -2),
                (-2, -1),
                (-2, 0),
                (-2, 1),
                (-2, 2),
                (-1, -2),
                (-1, 2),
                (0, -2),
                (0, 2),
                (1, -2),
                (1, 2),
                (2, -2),
                (2, -1),
                (2, 0),
                (2, 1),
                (2, 2),
            ],
        };

        let mut count = 0;
        for (dr, dc) in directions {
            let nr = r as isize + dr;
            let nc = c as isize + dc;

            if nr >= 0
                && nr < self.height as isize
                && nc >= 0
                && nc < self.width as isize
                && self.cells[nr as usize][nc as usize] == target_state_id
            {
                count += 1;
            }
        }
        count
    }
}
