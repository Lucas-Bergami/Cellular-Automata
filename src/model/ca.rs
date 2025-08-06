use rand::Rng;
// Repreuse rand::Rng;sents a single state in the CA
#[derive(Debug, Clone, PartialEq)]
pub struct CAState {
    pub id: u8, // Simple numeric ID, also used as index
    pub name: String,
    pub color: iced::Color,
}

// Relational operators for rules
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalOperator {
    Equals,
    NotEquals,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl RelationalOperator {
    pub const ALL: [RelationalOperator; 6] = [
        RelationalOperator::Equals,
        RelationalOperator::NotEquals,
        RelationalOperator::LessThan,
        RelationalOperator::LessOrEqual,
        RelationalOperator::GreaterThan,
        RelationalOperator::GreaterOrEqual,
    ];

    // Helper to evaluate the condition
    pub fn evaluate(&self, count: u8, threshold: u8) -> bool {
        match self {
            RelationalOperator::Equals => count == threshold,
            RelationalOperator::NotEquals => count != threshold,
            RelationalOperator::LessThan => count < threshold,
            RelationalOperator::LessOrEqual => count <= threshold,
            RelationalOperator::GreaterThan => count > threshold,
            RelationalOperator::GreaterOrEqual => count >= threshold,
        }
    }
}

impl std::fmt::Display for RelationalOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RelationalOperator::Equals => "==",
                RelationalOperator::NotEquals => "!=",
                RelationalOperator::LessThan => "<",
                RelationalOperator::LessOrEqual => "<=",
                RelationalOperator::GreaterThan => ">",
                RelationalOperator::GreaterOrEqual => ">=",
            }
        )
    }
}

// Represents a single transition rule
// Example: IF current_state_id is X AND (count of neighbors with Y is OP Z) THEN next_state_id is W
#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub current_state_id: u8,
    pub neighbor_state_id_to_count: u8, // The state of neighbors we are interested in counting
    pub operator: RelationalOperator,
    pub neighbor_count_threshold: u8, // The threshold for the count
    pub next_state_id: u8,
    // For display purposes primarily
    pub current_state_name: String,
    pub neighbor_state_name: String,
    pub next_state_name: String,
}

// The 2D grid for simulation
#[derive(Debug, Clone)]
pub struct CAGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<u8>>, // Stores state IDs
}

impl CAGrid {
    pub fn new(width: usize, height: usize) -> Self {
        //todo: mudar para gerar aleatoriamente de
        //acordo com os estados disponiveis e não somente 0 e 1
        let mut rng = rand::rng();

        let cells = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| if rng.random_bool(0.5) { 1 } else { 0 })
                    .collect()
            })
            .collect();

        CAGrid {
            width,
            height,
            cells,
        }
    }
    // Basic Moore neighborhood count
    pub fn count_neighbors(&self, r: usize, c: usize, target_state_id: u8) -> u8 {
        let mut count = 0;
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let nr = r as isize + dr;
                let nc = c as isize + dc;

                if nr >= 0 && nr < self.height as isize && nc >= 0 && nc < self.width as isize {
                    if self.cells[nr as usize][nc as usize] == target_state_id {
                        count += 1;
                    }
                }
                // Add boundary condition handling here (e.g., toroidal) if desired
            }
        }
        count
    }
    pub fn get_state(&self, r: usize, c: usize) -> Option<u8> {
        self.cells
            .get(r)
            .and_then(|row_vec| row_vec.get(c))
            .copied()
    }

    pub fn set_state(&mut self, r: usize, c: usize, state_id: u8) {
        if r < self.height && c < self.width {
            self.cells[r][c] = state_id;
        }
    }
}

// Need to implement Display for CAState for PickList
impl std::fmt::Display for CAState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.name, self.id)
    }
}
