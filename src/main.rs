use iced::widget::canvas::{self, Cache, Canvas, Geometry, Path, Stroke};
use iced::widget::pane_grid::mouse_interaction;
use iced::widget::text_input::cursor;
use iced::widget::{
    Button, Column, Container, PickList, Row, Scrollable, Slider, Text, TextInput, button, column,
    container, pick_list, row, scrollable, slider, text, text_input,
};
use iced::{
    Alignment, Color, Command, Element, Length, Point, Rectangle, Renderer, Settings, Size,
    Subscription, Theme, Vector, theme,
};
use iced::{Application, executor};
use std::fmt;
use std::time::{Duration, Instant};

// Represents a single state in the CA
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

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionCombiner {
    And,
    Or,
    Xor,
}

impl fmt::Display for ConditionCombiner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ConditionCombiner::And => "AND",
            ConditionCombiner::Or => "OR",
            ConditionCombiner::Xor => "XOR",
        };
        write!(f, "{}", s)
    }
}

impl ConditionCombiner {
    pub const ALL: [ConditionCombiner; 3] = [
        ConditionCombiner::And,
        ConditionCombiner::Or,
        ConditionCombiner::Xor,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExampleModel {
    GameOfLife,
    Wireworld,
    BrianBrain,
    TuringPatterns,
    ForestFire,
}

impl ExampleModel {
    pub const ALL: [ExampleModel; 5] = [
        ExampleModel::GameOfLife,
        ExampleModel::Wireworld,
        ExampleModel::BrianBrain,
        ExampleModel::TuringPatterns,
        ExampleModel::ForestFire,
    ];
}

impl std::fmt::Display for ExampleModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExampleModel::GameOfLife => write!(f, "Game of Life"),
            ExampleModel::Wireworld => write!(f, "Wireworld"),
            ExampleModel::BrianBrain => write!(f, "Brian’s Brain"),
            ExampleModel::TuringPatterns => write!(f, "Turing Patterns"),
            ExampleModel::ForestFire => write!(f, "Forest Fire"),
        }
    }
}

// Represents a single transition rule
#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub current_state_id: u8,

    pub neighbor_state_id_to_count: Vec<u8>,
    pub operator: Vec<RelationalOperator>,
    pub neighbor_count_threshold: Vec<u8>,
    pub combiner: Vec<ConditionCombiner>,

    pub next_state_id: u8,
    // For display
    pub current_state_name: String,
    pub neighbor_state_name: String,
    pub next_state_name: String,
}

impl TransitionRule {
    pub fn conditions_as_string(&self) -> String {
        let mut parts = Vec::new();
        for i in 0..self.neighbor_state_id_to_count.len() {
            let cond_str = format!(
                "count of '{}' {} {}",
                self.neighbor_state_name, self.operator[i], self.neighbor_count_threshold[i],
            );
            parts.push(cond_str);

            if i < self.combiner.len() {
                parts.push(format!("{:?}", self.combiner[i])); // AND / OR / XOR
            }
        }
        parts.join(" ")
    }
}

// The 2D grid for simulation
#[derive(Debug, Clone)]
pub struct CAGrid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<u8>>, // Stores state IDs
    pub neighborhood: Neighborhood,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub fn new(width: usize, height: usize, states: Vec<CAState>) -> Self {
        use rand::prelude::IndexedRandom;
        use rand::rng;

        let mut rng = rng();

        let available_state_ids: Vec<u8> = states.iter().map(|s| s.id).collect();

        let cells = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| *available_state_ids.choose(&mut rng).unwrap_or(&0))
                    .collect()
            })
            .collect();

        CAGrid {
            width,
            height,
            cells,
            neighborhood: Neighborhood::Moore,
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

            if nr >= 0 && nr < self.height as isize && nc >= 0 && nc < self.width as isize {
                if self.cells[nr as usize][nc as usize] == target_state_id {
                    count += 1;
                }
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

const DEFAULT_GRID_WIDTH: usize = 50;
const DEFAULT_GRID_HEIGHT: usize = 40;
const DEFAULT_STATE_ID: u8 = 1;

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

struct ConditionForm {
    neighbor_state: Option<CAState>,
    operator: Option<RelationalOperator>,
    threshold: String, // texto do input
    combiner: Option<ConditionCombiner>,
}

struct CASimulator {
    active_tab: TabId,
    states: Vec<CAState>,
    rules: Vec<TransitionRule>,
    grid: CAGrid,
    grid_cache: Cache,
    simulation_timer: Option<Instant>,
    is_simulating: bool,
    simulation_speed_ms: u64, // Milliseconds per step

    // --- UI Input State ---
    // State creation
    new_state_name: String,
    new_state_color_r: String, // Store as string for input, parse later
    new_state_color_g: String,
    new_state_color_b: String,

    // Rule creation
    rule_form_current_state: Option<CAState>,
    rule_form_neighbor_state: Option<CAState>,
    rule_form_operator: Option<RelationalOperator>,
    rule_form_threshold: String,
    rule_form_next_state: Option<CAState>,
    rule_form_error: Option<String>,
    rule_form_conditions: Vec<ConditionForm>,

    // Grid dimensions input
    grid_width_input: String,
    grid_height_input: String,

    // For picking next state on canvas click
    selected_paint_state_id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabId {
    Definition,
    Simulation,
}

#[derive(Debug, Clone)]
enum Message {
    TabSelected(TabId),
    Tick(Instant),

    // State definition
    StateNameChanged(String),
    StateColorRChanged(String),
    StateColorGChanged(String),
    StateColorBChanged(String),
    AddState,
    RemoveState(usize), // by index

    // Rule definition
    ExampleModelSelected(ExampleModel),
    RuleCombinerSelected(usize, ConditionCombiner),
    AddCondition,
    RemoveCondition(usize),
    RuleNeighborStateSelected(usize, CAState),
    RuleOperatorSelected(usize, RelationalOperator),
    RuleThresholdChanged(usize, String),
    RuleCurrentStateSelected(CAState),
    RuleNextStateSelected(CAState),
    AddRule,
    RemoveRule(usize), // by index

    // Grid/Simulation
    NeighborhoodChanged(Neighborhood),
    GridWidthChanged(String),
    GridHeightChanged(String),
    ApplyGridSize,
    ResetGrid,
    ToggleSimulation,
    NextStep,
    SimulationSpeedChanged(f32), // From slider (0-100), map to ms
    CanvasEvent(canvas::Event),  // To handle clicks on the canvas
    PaintStateSelected(CAState), // For selecting which state to paint on click
    PaintCell(usize, usize, u8),
}

impl Application for CASimulator {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let initial_states = vec![
            CAState {
                id: 0,
                name: "Dead".to_string(),
                color: Color::BLACK,
            },
            CAState {
                id: 1,
                name: "Alive".to_string(),
                color: Color::new(0.0, 1.0, 0.0, 1.0),
            },
        ];
        let grid = CAGrid::new(
            DEFAULT_GRID_WIDTH,
            DEFAULT_GRID_HEIGHT,
            initial_states.clone(),
        );
        let initial_rules = vec![
            // Alive -> Alive (if vizinhos == 2)
            TransitionRule {
                current_state_id: 1,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::Equals],
                neighbor_count_threshold: vec![2],
                combiner: vec![],
                next_state_id: 1,
                current_state_name: "Alive".into(),
                neighbor_state_name: "Alive".into(),
                next_state_name: "Alive".into(),
            },
            // Alive -> Alive (if vizinhos == 3)
            TransitionRule {
                current_state_id: 1,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::Equals],
                neighbor_count_threshold: vec![3],
                combiner: vec![],
                next_state_id: 1,
                current_state_name: "Alive".into(),
                neighbor_state_name: "Alive".into(),
                next_state_name: "Alive".into(),
            },
            // Dead -> Alive (if vizinhos == 3)
            TransitionRule {
                current_state_id: 0,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::Equals],
                neighbor_count_threshold: vec![3],
                combiner: vec![],
                next_state_id: 1,
                current_state_name: "Dead".into(),
                neighbor_state_name: "Alive".into(),
                next_state_name: "Alive".into(),
            },
            // Alive -> Dead (se vizinhos < 2)
            TransitionRule {
                current_state_id: 1,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::LessThan],
                neighbor_count_threshold: vec![2],
                combiner: vec![],
                next_state_id: 0,
                current_state_name: "Alive".into(),
                neighbor_state_name: "Alive".into(),
                next_state_name: "Dead".into(),
            },
            // Alive -> Dead (if vizinhos > 3)
            TransitionRule {
                current_state_id: 1,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::GreaterThan],
                neighbor_count_threshold: vec![3],
                combiner: vec![],
                next_state_id: 0,
                current_state_name: "Alive".into(),
                neighbor_state_name: "Alive".into(),
                next_state_name: "Dead".into(),
            },
        ];
        (
            CASimulator {
                active_tab: TabId::Definition,
                states: initial_states,
                rules: initial_rules,
                grid,
                grid_cache: Cache::new(),
                simulation_timer: None,
                is_simulating: false,
                simulation_speed_ms: 200, // Default speed

                new_state_name: String::new(),
                new_state_color_r: "0".to_string(),
                new_state_color_g: "0".to_string(),
                new_state_color_b: "0".to_string(),

                rule_form_current_state: None,
                rule_form_neighbor_state: None,
                rule_form_operator: Some(RelationalOperator::Equals),
                rule_form_threshold: "0".to_string(),
                rule_form_next_state: None,
                rule_form_error: None,
                rule_form_conditions: vec![],

                grid_width_input: DEFAULT_GRID_WIDTH.to_string(),
                grid_height_input: DEFAULT_GRID_HEIGHT.to_string(),
                selected_paint_state_id: DEFAULT_STATE_ID,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Cellular Automata Modeler")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::TabSelected(tab) => {
                self.active_tab = tab;
            }
            Message::Tick(_) => {
                if self.is_simulating {
                    self.step_simulation_logic();
                }
            }

            // --- State Definition Messages ---
            Message::StateNameChanged(name) => self.new_state_name = name,
            Message::StateColorRChanged(r) => self.new_state_color_r = r,
            Message::StateColorGChanged(g) => self.new_state_color_g = g,
            Message::StateColorBChanged(b) => self.new_state_color_b = b,
            Message::AddState => {
                if !self.new_state_name.trim().is_empty() {
                    let r = self.new_state_color_r.parse::<u8>().unwrap_or(0);
                    let g = self.new_state_color_g.parse::<u8>().unwrap_or(0);
                    let b = self.new_state_color_b.parse::<u8>().unwrap_or(0);

                    let mut new_id = 0u8;
                    let mut used_ids: Vec<u8> = self.states.iter().map(|s| s.id).collect();
                    used_ids.sort_unstable();
                    for id in used_ids {
                        if id == new_id {
                            new_id += 1;
                        } else if id > new_id {
                            break;
                        }
                    }

                    self.states.push(CAState {
                        id: new_id,
                        name: self.new_state_name.clone(),
                        color: Color::from_rgb8(r, g, b),
                    });

                    self.new_state_name.clear();
                }
            }
            Message::RemoveState(index) => {
                if index < self.states.len() {
                    let removed_state_id = self.states[index].id;
                    self.states.remove(index);
                    self.rules.retain(|rule| {
                        rule.current_state_id != removed_state_id
                            && !rule.neighbor_state_id_to_count.contains(&removed_state_id)
                            && rule.next_state_id != removed_state_id
                    });
                    for r in 0..self.grid.height {
                        for c in 0..self.grid.width {
                            if self.grid.cells[r][c] == removed_state_id {
                                self.grid.cells[r][c] = DEFAULT_STATE_ID;
                            }
                        }
                    }
                    self.grid_cache.clear();
                }
            }

            // --- Rule Definition Messages ---
            Message::RuleCurrentStateSelected(state) => self.rule_form_current_state = Some(state),
            Message::RuleNextStateSelected(state) => self.rule_form_next_state = Some(state),

            Message::ExampleModelSelected(model) => {
                self.states.clear();
                self.rules.clear();

                match model {
                    ExampleModel::GameOfLife => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Dead".into(),
                                color: Color::BLACK,
                            },
                            CAState {
                                id: 1,
                                name: "Alive".into(),
                                color: Color::from_rgb8(0, 255, 0),
                            },
                        ];

                        self.rules = vec![
                            TransitionRule {
                                current_state_id: 1,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::Equals],
                                neighbor_count_threshold: vec![2],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Alive".into(),
                                neighbor_state_name: "Alive".into(),
                                next_state_name: "Alive".into(),
                            },
                            TransitionRule {
                                current_state_id: 1,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::Equals],
                                neighbor_count_threshold: vec![3],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Alive".into(),
                                neighbor_state_name: "Alive".into(),
                                next_state_name: "Alive".into(),
                            },
                            TransitionRule {
                                current_state_id: 0,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::Equals],
                                neighbor_count_threshold: vec![3],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Dead".into(),
                                neighbor_state_name: "Alive".into(),
                                next_state_name: "Alive".into(),
                            },
                            TransitionRule {
                                current_state_id: 1,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::LessThan],
                                neighbor_count_threshold: vec![2],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Alive".into(),
                                neighbor_state_name: "Alive".into(),
                                next_state_name: "Dead".into(),
                            },
                            TransitionRule {
                                current_state_id: 1,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::GreaterThan],
                                neighbor_count_threshold: vec![3],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Alive".into(),
                                neighbor_state_name: "Alive".into(),
                                next_state_name: "Dead".into(),
                            },
                        ];
                    }

                    ExampleModel::Wireworld => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Empty".into(),
                                color: Color::BLACK,
                            },
                            CAState {
                                id: 1,
                                name: "Electron Head".into(),
                                color: Color::from_rgb8(0, 0, 255),
                            },
                            CAState {
                                id: 2,
                                name: "Electron Tail".into(),
                                color: Color::from_rgb8(255, 0, 0),
                            },
                            CAState {
                                id: 3,
                                name: "Conductor".into(),
                                color: Color::from_rgb8(255, 255, 0),
                            },
                        ];
                        self.rules = vec![
                            TransitionRule {
                                current_state_id: 1, // Head -> Tail
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 2,
                                current_state_name: "Electron Head".into(),
                                neighbor_state_name: "".into(),
                                next_state_name: "Electron Tail".into(),
                            },
                            TransitionRule {
                                current_state_id: 2, // Tail -> Conductor
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 3,
                                current_state_name: "Electron Tail".into(),
                                neighbor_state_name: "".into(),
                                next_state_name: "Conductor".into(),
                            },
                            TransitionRule {
                                current_state_id: 3, // Conductor -> Head if 1 or 2 vizinhos are Head
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![
                                    RelationalOperator::Equals,
                                    RelationalOperator::Equals,
                                ],
                                neighbor_count_threshold: vec![1, 2],
                                combiner: vec![ConditionCombiner::Or],
                                next_state_id: 1,
                                current_state_name: "Conductor".into(),
                                neighbor_state_name: "Electron Head".into(),
                                next_state_name: "Electron Head".into(),
                            },
                        ];
                    }

                    ExampleModel::BrianBrain => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Off".into(),
                                color: Color::BLACK,
                            },
                            CAState {
                                id: 1,
                                name: "On".into(),
                                color: Color::from_rgb8(0, 0, 255),
                            },
                            CAState {
                                id: 2,
                                name: "Dying".into(),
                                color: Color::from_rgb8(255, 0, 0),
                            },
                        ];

                        self.rules = vec![
                            TransitionRule {
                                current_state_id: 0, // Off -> On if 2 vizinhos are On
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::Equals],
                                neighbor_count_threshold: vec![2],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Off".into(),
                                neighbor_state_name: "On".into(),
                                next_state_name: "On".into(),
                            },
                            TransitionRule {
                                current_state_id: 1, // On -> Dying
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 2,
                                current_state_name: "On".into(),
                                neighbor_state_name: "".into(),
                                next_state_name: "Dying".into(),
                            },
                            TransitionRule {
                                current_state_id: 2, // Dying -> Off
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Dying".into(),
                                neighbor_state_name: "".into(),
                                next_state_name: "Off".into(),
                            },
                        ];
                    }

                    ExampleModel::TuringPatterns => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Empty".into(),
                                color: Color::BLACK,
                            },
                            CAState {
                                id: 1,
                                name: "Activator".into(),
                                color: Color::from_rgb8(0, 200, 255),
                            },
                            CAState {
                                id: 2,
                                name: "Inhibitor".into(),
                                color: Color::from_rgb8(255, 100, 0),
                            },
                        ];

                        self.rules = vec![
                            TransitionRule {
                                current_state_id: 0, // Empty -> Activator if >=2 vizinhos Activator
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::GreaterOrEqual],
                                neighbor_count_threshold: vec![2],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Empty".into(),
                                neighbor_state_name: "Activator".into(),
                                next_state_name: "Activator".into(),
                            },
                            TransitionRule {
                                current_state_id: 1, // Activator -> Inhibitor if >=3 vizinhos Activator
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::GreaterOrEqual],
                                neighbor_count_threshold: vec![3],
                                combiner: vec![],
                                next_state_id: 2,
                                current_state_name: "Activator".into(),
                                neighbor_state_name: "Activator".into(),
                                next_state_name: "Inhibitor".into(),
                            },
                            TransitionRule {
                                current_state_id: 2, // Inhibitor -> Empty
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Inhibitor".into(),
                                neighbor_state_name: "".into(),
                                next_state_name: "Empty".into(),
                            },
                        ];
                    }

                    ExampleModel::ForestFire => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Empty".into(),
                                color: Color::BLACK,
                            },
                            CAState {
                                id: 1,
                                name: "Tree".into(),
                                color: Color::from_rgb8(0, 200, 0),
                            },
                            CAState {
                                id: 2,
                                name: "Burning".into(),
                                color: Color::from_rgb8(255, 0, 0),
                            },
                        ];

                        self.rules = vec![
                            TransitionRule {
                                current_state_id: 2, // Burning -> Empty
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Burning".into(),
                                neighbor_state_name: "".into(),
                                next_state_name: "Empty".into(),
                            },
                            TransitionRule {
                                current_state_id: 1, // Tree -> Burning if >=1 vizinho Burning
                                neighbor_state_id_to_count: vec![2],
                                operator: vec![RelationalOperator::GreaterOrEqual],
                                neighbor_count_threshold: vec![1],
                                combiner: vec![],
                                next_state_id: 2,
                                current_state_name: "Tree".into(),
                                neighbor_state_name: "Burning".into(),
                                next_state_name: "Burning".into(),
                            },
                            TransitionRule {
                                current_state_id: 0, // Empty -> Tree (budding)
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Empty".into(),
                                neighbor_state_name: "".into(),
                                next_state_name: "Tree".into(),
                            },
                        ];
                    }
                }

                self.grid_cache.clear();
            }
            Message::RuleCombinerSelected(idx, comb) => {
                if idx < self.rule_form_conditions.len() {
                    self.rule_form_conditions[idx].combiner = Some(comb);
                }
            }

            Message::AddCondition => {
                self.rule_form_conditions.push(ConditionForm {
                    neighbor_state: None,
                    operator: None,
                    threshold: String::new(),
                    combiner: None,
                });
            }
            Message::RemoveCondition(idx) => {
                if idx < self.rule_form_conditions.len() {
                    self.rule_form_conditions.remove(idx);
                }
            }
            Message::RuleNeighborStateSelected(idx, state) => {
                if idx < self.rule_form_conditions.len() {
                    self.rule_form_conditions[idx].neighbor_state = Some(state);
                }
            }
            Message::RuleOperatorSelected(idx, op) => {
                if idx < self.rule_form_conditions.len() {
                    self.rule_form_conditions[idx].operator = Some(op);
                }
            }
            Message::RuleThresholdChanged(idx, val) => {
                if idx < self.rule_form_conditions.len() {
                    self.rule_form_conditions[idx].threshold = val;
                }
            }

            Message::AddRule => {
                self.rule_form_error = None;
                let mut errors: Vec<String> = Vec::new();

                // Current state
                let cur = if let Some(s) = self.rule_form_current_state.as_ref() {
                    s
                } else {
                    errors.push("Current State não selecionado".to_string());
                    &CAState {
                        id: 0,
                        name: "".into(),
                        color: Color::WHITE,
                    }
                };

                // Next state
                let nxt = if let Some(s) = self.rule_form_next_state.as_ref() {
                    s
                } else {
                    errors.push("Next State não selecionado".to_string());
                    &CAState {
                        id: 0,
                        name: "".into(),
                        color: Color::WHITE,
                    }
                };

                if self.rule_form_conditions.is_empty() {
                    errors.push("Nenhuma condição adicionada à regra".to_string());
                }

                let mut neighbor_ids: Vec<u8> = Vec::new();
                let mut operators: Vec<RelationalOperator> = Vec::new();
                let mut thresholds: Vec<u8> = Vec::new();
                let mut combiners: Vec<ConditionCombiner> = Vec::new();

                for (idx, cond) in self.rule_form_conditions.iter().enumerate() {
                    // Neighbor state
                    if let Some(state) = &cond.neighbor_state {
                        neighbor_ids.push(state.id);
                    } else {
                        errors.push(format!(
                            "Neighbor State não selecionado na condição {}",
                            idx + 1
                        ));
                        neighbor_ids.push(0);
                    }

                    // Operator
                    if let Some(op) = cond.operator {
                        operators.push(op);
                    } else {
                        errors.push(format!("Operador não selecionado na condição {}", idx + 1));
                        operators.push(RelationalOperator::Equals);
                    }

                    // Threshold
                    match cond.threshold.parse::<u8>() {
                        Ok(v) => thresholds.push(v),
                        Err(_) => {
                            errors.push(format!("Threshold inválido na condição {}", idx + 1));
                            thresholds.push(0);
                        }
                    }

                    if idx < self.rule_form_conditions.len() - 1 {
                        if let Some(comb) = cond.combiner.clone() {
                            combiners.push(comb);
                        } else {
                            combiners.push(ConditionCombiner::And);
                        }
                    }
                }

                if !errors.is_empty() {
                    self.rule_form_error = Some(errors.join("; "));
                } else {
                    self.rules.push(TransitionRule {
                        current_state_id: cur.id,
                        neighbor_state_id_to_count: neighbor_ids,
                        operator: operators,
                        neighbor_count_threshold: thresholds,
                        combiner: combiners,
                        next_state_id: nxt.id,
                        current_state_name: cur.name.clone(),
                        neighbor_state_name: String::new(),
                        next_state_name: nxt.name.clone(),
                    });

                    // Reset form
                    self.rule_form_current_state = None;
                    self.rule_form_next_state = None;
                    self.rule_form_conditions.clear();
                    self.rule_form_error = None;
                }
            }

            Message::RemoveRule(idx) => {
                if idx < self.rules.len() {
                    self.rules.remove(idx);
                }
            }

            // --- Grid/Simulation Messages ---
            Message::NeighborhoodChanged(nb) => self.grid.neighborhood = nb,
            Message::GridWidthChanged(w) => self.grid_width_input = w,
            Message::GridHeightChanged(h) => self.grid_height_input = h,
            Message::ApplyGridSize => {
                let width = self.grid_width_input.parse().unwrap_or(DEFAULT_GRID_WIDTH);
                let height = self
                    .grid_height_input
                    .parse()
                    .unwrap_or(DEFAULT_GRID_HEIGHT);
                self.grid = CAGrid::new(width, height, self.states.clone());
                self.grid_cache.clear();
            }
            Message::ResetGrid => {
                self.grid = CAGrid::new(self.grid.width, self.grid.height, self.states.clone());
                self.grid_cache.clear();
            }
            Message::ToggleSimulation => {
                self.is_simulating = !self.is_simulating;
                self.simulation_timer = if self.is_simulating {
                    Some(Instant::now())
                } else {
                    None
                };
            }
            Message::NextStep => self.step_simulation_logic(),
            Message::SimulationSpeedChanged(value) => {
                let inv_value = 100.0 - value;
                self.simulation_speed_ms = (10.0 + inv_value * 9.9) as u64;
            }
            Message::CanvasEvent(event) => {
                if let canvas::Event::Mouse(mouse_event) = event {
                    if mouse_event == iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) {
                        // lógica de clique opcional
                    }
                }
            }
            Message::PaintStateSelected(state) => {
                self.selected_paint_state_id = state.id;
                println!(
                    "Cor selecionada: R={} G={} B={}",
                    state.color.r, state.color.g, state.color.b
                );
            }
            Message::PaintCell(row, col, state_id) => {
                self.grid.cells[row][col] = state_id;
                self.grid_cache.clear();
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let header = text("Cellular Automata Modeler").size(30);

        let tab_buttons = row![
            button(text("Define Model"))
                .on_press(Message::TabSelected(TabId::Definition))
                .style(if self.active_tab == TabId::Definition {
                    theme::Button::Primary
                } else {
                    theme::Button::Secondary
                }),
            button(text("Simulate"))
                .on_press(Message::TabSelected(TabId::Simulation))
                .style(if self.active_tab == TabId::Simulation {
                    theme::Button::Primary
                } else {
                    theme::Button::Secondary
                }),
            // button(text("Model Image")).on_press(Message::TabSelected(TabId::ModelImage))
            //     .style(if self.active_tab == TabId::ModelImage { theme::Button::Primary } else { theme::Button::Secondary }),
        ]
        .spacing(10);

        let content = match self.active_tab {
            TabId::Definition => self.view_definition_tab(),
            TabId::Simulation => self.view_simulation_tab(),
            // TabId::ModelImage => self.view_model_image_tab(),
        };

        column![header, tab_buttons, content]
            .spacing(20)
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark // Or Theme::Light, or a custom one
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.is_simulating {
            iced::time::every(Duration::from_millis(self.simulation_speed_ms)).map(Message::Tick)
        } else {
            Subscription::none()
        }
    }
}

impl CASimulator {
    fn view_definition_tab(&self) -> Element<Message> {
        let model_selector = column![
            text("Load Example Model").size(20),
            PickList::new(
                ExampleModel::ALL.to_vec(),
                None::<ExampleModel>,
                Message::ExampleModelSelected,
            )
            .placeholder("Select a model"),
        ]
        .spacing(10)
        .width(Length::Fill);
        // --- State Creation Panel ---
        let state_creation_panel = column![
            text("Create New State").size(20),
            text_input("State Name (e.g., Alive)", &self.new_state_name)
                .on_input(Message::StateNameChanged)
                .padding(5),
            row![
                text("R:"),
                text_input("0-255", &self.new_state_color_r)
                    .on_input(Message::StateColorRChanged)
                    .padding(3)
                    .width(Length::Fixed(60.0)),
                text("G:"),
                text_input("0-255", &self.new_state_color_g)
                    .on_input(Message::StateColorGChanged)
                    .padding(3)
                    .width(Length::Fixed(60.0)),
                text("B:"),
                text_input("0-255", &self.new_state_color_b)
                    .on_input(Message::StateColorBChanged)
                    .padding(3)
                    .width(Length::Fixed(60.0)),
            ]
            .spacing(5)
            .align_items(Alignment::Center),
            button("Add State").on_press(Message::AddState).padding(5),
        ]
        .spacing(10)
        .width(Length::Fill);

        let states_list: Element<Message> = if self.states.is_empty() {
            text("No states defined yet.").into()
        } else {
            self.states
                .iter()
                .enumerate()
                .fold(
                    Column::new().spacing(5).width(Length::Fill),
                    |col, (idx, state)| {
                        col.push(
                            row![
                                text(format!("{}: {}", state.id, state.name)).width(Length::Fill),
                                button(text("Remove"))
                                    .on_press(Message::RemoveState(idx))
                                    .style(theme::Button::Destructive)
                                    .padding(5),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                        )
                    },
                )
                .into()
        };
        let states_panel = column![
            text("Defined States").size(20),
            Scrollable::new(states_list)
                .height(Length::Fixed(150.0))
                .width(Length::Fill),
        ]
        .spacing(10)
        .width(Length::Fill);

        // --- Rule Creation Panel ---
        let available_states_for_picklist = self.states.clone();

        let mut rule_creation_panel = column![
            text("Create New Transition Rule").size(20),
            // Current State
            text("IF Current State is:"),
            PickList::new(
                available_states_for_picklist.clone(),
                self.rule_form_current_state.clone(),
                Message::RuleCurrentStateSelected,
            )
            .placeholder("Select Current State"),
            // Condições
            text("AND the following conditions are met:"),
        ];

        // Loop sobre as condições
        for idx in 0..self.rule_form_conditions.len() {
            let cond = &self.rule_form_conditions[idx];

            let mut condition_row = row![
                // Neighbor State
                PickList::new(
                    available_states_for_picklist.clone(),
                    cond.neighbor_state.clone(),
                    {
                        let idx = idx;
                        move |s| Message::RuleNeighborStateSelected(idx, s)
                    }
                )
                .placeholder("Neighbor State"),
                // Operator
                PickList::new(RelationalOperator::ALL.to_vec(), cond.operator, {
                    let idx = idx;
                    move |op| Message::RuleOperatorSelected(idx, op)
                })
                .placeholder("Operator"),
                // Threshold
                text_input("Count (e.g., 3)", &cond.threshold)
                    .on_input({
                        let idx = idx;
                        move |val| Message::RuleThresholdChanged(idx, val)
                    })
                    .padding(5)
                    .width(Length::Fixed(80.0)),
                // Remove condition button
                button("-").on_press(Message::RemoveCondition(idx))
            ]
            .spacing(5);

            if idx < self.rule_form_conditions.len() - 1 {
                condition_row = condition_row.push(
                    PickList::new(ConditionCombiner::ALL.to_vec(), cond.combiner.clone(), {
                        let idx = idx;
                        move |comb| Message::RuleCombinerSelected(idx, comb)
                    })
                    .placeholder("Combiner")
                    .width(Length::Fixed(80.0)),
                );
            }

            rule_creation_panel = rule_creation_panel.push(condition_row);
        }

        rule_creation_panel = rule_creation_panel.push(
            button("+ Add Condition")
                .on_press(Message::AddCondition)
                .padding(5),
        );

        // Next State
        rule_creation_panel = rule_creation_panel.push(text("THEN Next State is:")).push(
            PickList::new(
                available_states_for_picklist.clone(),
                self.rule_form_next_state.clone(),
                Message::RuleNextStateSelected,
            )
            .placeholder("Select Next State"),
        );

        rule_creation_panel =
            rule_creation_panel.push(button("Add Rule").on_press(Message::AddRule).padding(5));

        if let Some(err) = &self.rule_form_error {
            rule_creation_panel =
                rule_creation_panel.push(text(err).size(16).style(Color::from_rgb8(255, 0, 0)));
        }

        let rules_list: Element<Message> = if self.rules.is_empty() {
            text("No rules defined yet.").into()
        } else {
            self.rules
                .iter()
                .enumerate()
                .fold(
                    Column::new().spacing(5).width(Length::Fill),
                    |col, (idx, rule)| {
                        col.push(
                            row![
                                text(format!(
                                    "IF current is '{}' {} THEN next is '{}'", //TODO:FIX
                                    rule.current_state_name,
                                    rule.conditions_as_string(),
                                    rule.next_state_name
                                ))
                                .width(Length::Fill),
                                button(text("Remove"))
                                    .on_press(Message::RemoveRule(idx))
                                    .style(theme::Button::Destructive)
                                    .padding(5),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                        )
                    },
                )
                .into()
        };
        let rules_panel = column![
            text("Defined Rules").size(20),
            Scrollable::new(rules_list)
                .height(Length::Fixed(200.0))
                .width(Length::Fill),
        ]
        .spacing(10)
        .width(Length::Fill);

        Scrollable::new(
            Container::new(
                column![
                    model_selector,
                    iced::widget::horizontal_rule(10),
                    state_creation_panel,
                    iced::widget::horizontal_rule(10),
                    states_panel,
                    iced::widget::horizontal_rule(10),
                    rule_creation_panel,
                    iced::widget::horizontal_rule(10),
                    rules_panel,
                ]
                .spacing(20)
                .padding([0, 15, 0, 0])
                .width(Length::Fill)
                .align_items(Alignment::Start),
            )
            .padding([0, 0, 15, 0])
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .into()
    }

    fn view_simulation_tab(&self) -> Element<Message> {
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
            PickList::new(
                vec![
                    Neighborhood::VonNeumann,
                    Neighborhood::Moore,
                    Neighborhood::ExtendedMoore
                ],
                Some(self.grid.neighborhood),
                Message::NeighborhoodChanged
            )
            .placeholder("Select Neighborhood"),
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

    fn step_simulation_logic(&mut self) {
        if self.states.is_empty() {
            return;
        } // No states, nothing to do

        let mut next_grid_cells = self.grid.cells.clone();
        let current_grid = &self.grid; // Immutable borrow for reading

        for r in 0..current_grid.height {
            for c in 0..current_grid.width {
                let current_cell_state_id = current_grid.cells[r][c];
                let mut new_state_id = current_cell_state_id; // Default to no change

                println!(
                    "Celula ({}, {}) estado atual: {}",
                    r, c, current_cell_state_id
                );

                for (rule_idx, rule) in self.rules.iter().enumerate() {
                    if rule.current_state_id == current_cell_state_id {
                        println!(
                            "  Testando regra {} -> next {}",
                            rule_idx, rule.next_state_id
                        );

                        let mut results = Vec::new();

                        for i in 0..rule.neighbor_state_id_to_count.len() {
                            let neighbor_state = rule.neighbor_state_id_to_count[i];
                            let op = rule.operator[i];
                            let thr = rule.neighbor_count_threshold[i];

                            let neighbor_count = current_grid.count_neighbors(r, c, neighbor_state);
                            let res = op.evaluate(neighbor_count, thr);

                            println!(
                                "    Condição {}: count of {} = {} {} {} ? {}",
                                i, neighbor_state, neighbor_count, op, thr, res
                            );

                            results.push(res);
                        }

                        // If there are no conditions, consider it automatically true
                        let final_result = if results.is_empty() {
                            true
                        } else {
                            let mut res = results[0];
                            for i in 1..results.len() {
                                let combiner = &rule.combiner[i - 1];
                                let before = res;
                                res = match combiner {
                                    ConditionCombiner::And => res && results[i],
                                    ConditionCombiner::Or => res || results[i],
                                    ConditionCombiner::Xor => res ^ results[i],
                                };
                                println!(
                                    "    Combinação {}: {:?} entre {} e {} = {}",
                                    i, combiner, before, results[i], res
                                );
                            }
                            res
                        };

                        println!("  Resultado final da regra {}: {}", rule_idx, final_result);

                        if final_result {
                            println!(
                                "  >>> Regra {} aplicada: célula muda de {} para {}",
                                rule_idx, current_cell_state_id, rule.next_state_id
                            );
                            new_state_id = rule.next_state_id;
                            break;
                        }
                    }
                }

                next_grid_cells[r][c] = new_state_id;
            }
        }

        self.grid.cells = next_grid_cells;
        self.grid_cache.clear(); // Crucial: Invalidate cache to force redraw
    }
}

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

            let cell_width = frame.width() / self.grid.width as f32;
            let cell_height = frame.height() / self.grid.height as f32;

            for r in 0..self.grid.height {
                for c in 0..self.grid.width {
                    let state_id = self.grid.cells[r][c];
                    let cell_color = self
                        .states
                        .iter()
                        .find(|s| s.id == state_id)
                        .map_or(Color::new(1.0, 0.0, 0.0, 1.0), |s| s.color); // Magenta for unknown state

                    let top_left = Point::new(c as f32 * cell_width, r as f32 * cell_height);
                    let size = Size::new(cell_width, cell_height);
                    frame.fill_rectangle(top_left, size, cell_color);

                    // Optional: Draw grid lines if cells are large enough
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
            // Pega a posição do cursor relativa ao canvas
            if let Some(position) = cursor.position_in(bounds) {
                if self.grid.width > 0 && self.grid.height > 0 {
                    let cell_width = bounds.width / self.grid.width as f32;
                    let cell_height = bounds.height / self.grid.height as f32;

                    let col = (position.x / cell_width) as usize;
                    let row = (position.y / cell_height) as usize;

                    println!("Clique detectado em célula: linha {}, coluna {}", row, col);

                    if row < self.grid.height && col < self.grid.width {
                        return (
                            canvas::event::Status::Captured,
                            Some(Message::PaintCell(row, col, self.selected_paint_state_id)),
                        );
                    }
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

// Need to implement Display for CAState for PickList
impl std::fmt::Display for CAState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (ID: {})", self.name, self.id)
    }
}

// TODO: allow creating a rule without any conditions
