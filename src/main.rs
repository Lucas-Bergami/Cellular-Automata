use iced::widget::canvas::{self, Cache, Canvas, Geometry, Path, Stroke};
use iced::widget::pane_grid::mouse_interaction;
use iced::widget::text_input::cursor;
use iced::widget::{
    button, column, container, pick_list, row, scrollable, slider, text, text_input, Button,
    Column, Container, PickList, Row, Scrollable, Slider, Text, TextInput,
};
use iced::{executor, Application};
use iced::{
    theme, Alignment, Color, Command, Element, Length, Point, Rectangle, Renderer, Settings, Size,
    Subscription, Theme, Vector,
};
use std::time::{Duration, Instant};

// --- Paste CAState, RelationalOperator, TransitionRule, CAGrid structs here ---
// (from the section above)

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

// Represents a single transition rule
#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub current_state_id: u8,
    pub neighbor_state_id_to_count: u8,
    pub operator: RelationalOperator,
    pub neighbor_count_threshold: u8,
    pub next_state_id: u8,
    // For display
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
    pub fn new(width: usize, height: usize, default_state_id: u8) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let cells = (0..height)
            .map(|_| {
                (0..width)
                    .map(|_| if rng.gen_bool(0.5) { 1 } else { 0 }) // escolhe aleatório 0 ou 1
                    .collect()
            })
            .collect();

        CAGrid {
            width,
            height,
            cells,
        }
    }

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

// --- End of pasted structs ---

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
    // ModelImage, // You mentioned "image of the created model"
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
    RuleCurrentStateSelected(CAState),
    RuleNeighborStateSelected(CAState),
    RuleOperatorSelected(RelationalOperator),
    RuleThresholdChanged(String),
    RuleNextStateSelected(CAState),
    AddRule,
    RemoveRule(usize), // by index

    // Grid/Simulation
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
        let grid = CAGrid::new(DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT, DEFAULT_STATE_ID);

        (
            CASimulator {
                active_tab: TabId::Definition,
                states: initial_states,
                rules: Vec::new(),
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
                    let new_id = self.states.len() as u8; // Simple ID assignment
                    self.states.push(CAState {
                        id: new_id,
                        name: self.new_state_name.clone(),
                        color: Color::from_rgb8(r, g, b),
                    });
                    self.new_state_name.clear();
                    // Optionally reset color inputs
                }
            }
            Message::RemoveState(index) => {
                if index < self.states.len() {
                    let removed_state_id = self.states[index].id;
                    self.states.remove(index);
                    // Important: Need to update rules that might reference this state,
                    // or prevent removal if state is in use by rules/grid.
                    // For simplicity, we are not doing that here, which can lead to crashes.
                    // A robust solution would re-index or validate.
                    self.rules.retain(|rule| {
                        rule.current_state_id != removed_state_id
                            && rule.neighbor_state_id_to_count != removed_state_id
                            && rule.next_state_id != removed_state_id
                    });
                    // Reset any cells on the grid that used this state to default
                    for r in 0..self.grid.height {
                        for c in 0..self.grid.width {
                            if self.grid.cells[r][c] == removed_state_id {
                                self.grid.cells[r][c] = DEFAULT_STATE_ID;
                            }
                        }
                    }
                    self.grid_cache.clear(); // Grid needs redraw
                }
            }

            // --- Rule Definition Messages ---
            Message::RuleCurrentStateSelected(state) => self.rule_form_current_state = Some(state),
            Message::RuleNeighborStateSelected(state) => {
                self.rule_form_neighbor_state = Some(state)
            }
            Message::RuleOperatorSelected(op) => self.rule_form_operator = Some(op),
            Message::RuleThresholdChanged(val) => self.rule_form_threshold = val,
            Message::RuleNextStateSelected(state) => self.rule_form_next_state = Some(state),
            Message::AddRule => {
                // Limpa erro anterior
                self.rule_form_error = None;

                // Valida cada pedaço com feedback específico
                let mut errors = Vec::new();

                let cur = if let Some(s) = self.rule_form_current_state.as_ref() {
                    s
                } else {
                    errors.push("Current State não selecionado");
                    // placeholder, não importa porque vamos abortar se tiver erro
                    &CAState {
                        id: 0,
                        name: "".into(),
                        color: Color::WHITE,
                    }
                };

                let ngh = if let Some(s) = self.rule_form_neighbor_state.as_ref() {
                    s
                } else {
                    errors.push("Neighbor State não selecionado");
                    &CAState {
                        id: 0,
                        name: "".into(),
                        color: Color::WHITE,
                    }
                };

                let op = if let Some(o) = self.rule_form_operator {
                    o
                } else {
                    errors.push("Operator não selecionado");
                    RelationalOperator::Equals // valor fictício
                };

                let thr = match self.rule_form_threshold.parse::<u8>() {
                    Ok(v) => v,
                    Err(_) => {
                        errors.push("Threshold inválido (deve ser número entre 0 e 255)");
                        0
                    }
                };

                let nxt = if let Some(s) = self.rule_form_next_state.as_ref() {
                    s
                } else {
                    errors.push("Next State não selecionado");
                    &CAState {
                        id: 0,
                        name: "".into(),
                        color: Color::WHITE,
                    }
                };

                if !errors.is_empty() {
                    self.rule_form_error = Some(errors.join("; "));
                } else {
                    self.rules.push(TransitionRule {
                        current_state_id: cur.id,
                        neighbor_state_id_to_count: ngh.id,
                        operator: op,
                        neighbor_count_threshold: thr,
                        next_state_id: nxt.id,
                        current_state_name: cur.name.clone(),
                        neighbor_state_name: ngh.name.clone(),
                        next_state_name: nxt.name.clone(),
                    });
                    // opcional: limpar formulário
                    self.rule_form_current_state = None;
                    self.rule_form_neighbor_state = None;
                    self.rule_form_operator = None;
                    self.rule_form_threshold.clear();
                    self.rule_form_next_state = None;
                    self.rule_form_error = None;
                }
            }
            Message::RemoveRule(index) => {
                if index < self.rules.len() {
                    self.rules.remove(index);
                }
            }

            // --- Grid/Simulation Messages ---
            Message::GridWidthChanged(w) => self.grid_width_input = w,
            Message::GridHeightChanged(h) => self.grid_height_input = h,
            Message::ApplyGridSize => {
                let width = self.grid_width_input.parse().unwrap_or(DEFAULT_GRID_WIDTH);
                let height = self
                    .grid_height_input
                    .parse()
                    .unwrap_or(DEFAULT_GRID_HEIGHT);
                self.grid = CAGrid::new(width, height, DEFAULT_STATE_ID);
                self.grid_cache.clear();
            }
            Message::ResetGrid => {
                self.grid = CAGrid::new(self.grid.width, self.grid.height, DEFAULT_STATE_ID);
                self.grid_cache.clear();
            }
            Message::ToggleSimulation => {
                self.is_simulating = !self.is_simulating;
                if self.is_simulating {
                    self.simulation_timer = Some(Instant::now());
                } else {
                    self.simulation_timer = None;
                }
            }
            Message::NextStep => {
                self.step_simulation_logic();
            }
            Message::SimulationSpeedChanged(value) => {
                // Slider 0-100. Map to e.g. 1000ms to 10ms
                // (100 - value) makes slider left slow, right fast
                let inv_value = 100.0 - value;
                self.simulation_speed_ms = (10.0 + inv_value * 9.9) as u64; // Maps 0-100 to 1000ms-10ms
            }
            Message::CanvasEvent(event) => {
                if let canvas::Event::Mouse(mouse_event) = event {
                    if mouse_event == iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) {
                        /*if let Some(cursor_pos) = cursor.position {
                             // Determine cell size (approximate for now)
                            let cell_width = if self.grid.width > 0 { 600.0 / self.grid.width as f32 } else { 10.0 };
                            let cell_height = if self.grid.height > 0 { 600.0 / self.grid.height as f32 } else { 10.0 };

                            let c = (cursor_pos.x / cell_width).floor() as usize;
                            let r = (cursor_pos.y / cell_height).floor() as usize;

                            if r < self.grid.height && c < self.grid.width {
                                self.grid.set_state(r, c, self.selected_paint_state_id);
                                self.grid_cache.clear(); // Redraw
                            }
                        }*/
                    }
                }
            }
            Message::PaintStateSelected(selected_state) => {
                println!(
                    "Cor selecionada: R={} G={} B={}",
                    selected_state.color.r, selected_state.color.g, selected_state.color.b,
                );
                self.selected_paint_state_id = selected_state.id;
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
        let available_states_for_picklist = self.states.clone(); // Clone for pick_list

        let mut rule_creation_panel = column![
            text("Create New Transition Rule").size(20),
            text("IF Current State is:"),
            PickList::new(
                available_states_for_picklist.clone(),
                self.rule_form_current_state.clone(),
                Message::RuleCurrentStateSelected,
            )
            .placeholder("Select Current State"),
            text("AND Count of Neighbors with State:"),
            PickList::new(
                available_states_for_picklist.clone(),
                self.rule_form_neighbor_state.clone(),
                Message::RuleNeighborStateSelected,
            )
            .placeholder("Select Neighbor State to Count"),
            text("Is:"),
            row![
                PickList::new(
                    RelationalOperator::ALL.to_vec(),
                    self.rule_form_operator,
                    Message::RuleOperatorSelected
                )
                .placeholder("Operator"),
                text_input("Count (e.g., 3)", &self.rule_form_threshold)
                    .on_input(Message::RuleThresholdChanged)
                    .padding(5)
                    .width(Length::Fixed(80.0)),
            ]
            .spacing(5),
            text("THEN Next State is:"),
            PickList::new(
                available_states_for_picklist, // consumido aqui
                self.rule_form_next_state.clone(),
                Message::RuleNextStateSelected,
            )
            .placeholder("Select Next State"),
            button("Add Rule").on_press(Message::AddRule).padding(5),
        ]
        .spacing(10)
        .width(Length::Fill);

        // injeta feedback de erro se existir
        if let Some(err) = &self.rule_form_error {
            rule_creation_panel = rule_creation_panel.push(text(err).size(16));
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
                                    "IF current is '{}' AND count of '{}' {} {} THEN next is '{}'",
                                    rule.current_state_name,
                                    rule.neighbor_state_name,
                                    rule.operator,
                                    rule.neighbor_count_threshold,
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
                    state_creation_panel,
                    iced::widget::horizontal_rule(10),
                    states_panel,
                    iced::widget::horizontal_rule(10),
                    rule_creation_panel,
                    iced::widget::horizontal_rule(10),
                    rules_panel,
                ]
                .spacing(20)
                .padding([0, 15, 0, 0]) // espaçamento à direita para “abrir espaço” da scrollbar
                .width(Length::Fill)
                .align_items(Alignment::Start),
            )
            .padding([0, 0, 15, 0]) // espaçamento à direita para “abrir espaço” da scrollbar
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
        // Detecta se foi um evento de clique do mouse
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
                            Some(Message::PaintCell(
                                row,
                                col,
                                self.selected_paint_state_id, // estado vindo do PickList
                            )),
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
