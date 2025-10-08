#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExampleModel {
    GameOfLife,
    Wireworld,
    Greenberg,
    TuringPatterns,
    ForestFire,
}

impl ExampleModel {
    pub const ALL: [ExampleModel; 5] = [
        ExampleModel::GameOfLife,
        ExampleModel::Wireworld,
        ExampleModel::Greenberg,
        ExampleModel::TuringPatterns,
        ExampleModel::ForestFire,
    ];
}

impl std::fmt::Display for ExampleModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExampleModel::GameOfLife => write!(f, "Game of Life"),
            ExampleModel::Wireworld => write!(f, "Wireworld"),
            ExampleModel::Greenberg => write!(f, "Greenberg"),
            ExampleModel::TuringPatterns => write!(f, "Turing Patterns"),
            ExampleModel::ForestFire => write!(f, "Forest Fire"),
        }
    }
}
