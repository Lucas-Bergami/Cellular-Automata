#[derive(Debug, Clone, PartialEq)]
pub struct CAState {
    pub id: u8,
    pub name: String,
    pub color: iced::Color,
    pub weight: u8,
}

impl std::fmt::Display for CAState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.name, self.id)
    }
}
