use crate::controller::message::Message;
use crate::model::ca::{CAGrid, CAState, RelationalOperator, TransitionRule};
use iced::theme;
use iced::widget::canvas::Cache;
use iced::widget::{button, canvas, column, row, text};
use iced::{executor, Application, Color, Command, Element, Subscription, Theme};
use std::time::{Duration, Instant};

const DEFAULT_GRID_WIDTH: usize = 50;
const DEFAULT_GRID_HEIGHT: usize = 40;
const DEFAULT_STATE_ID: u8 = 1;

pub struct CASimulator {
    pub active_tab: TabId,
    pub states: Vec<CAState>,
    pub rules: Vec<TransitionRule>,
    pub grid: CAGrid,
    pub grid_cache: Cache,
    pub simulation_timer: Option<Instant>,
    pub is_simulating: bool,
    pub simulation_speed_ms: u64, // Milliseconds per step

    // --- UI Input State ---
    // State creation
    pub new_state_name: String,
    pub new_state_color_r: String, // Store as string for input, parse later
    pub new_state_color_g: String,
    pub new_state_color_b: String,

    // Rule creation
    pub rule_form_current_state: Option<CAState>,
    pub rule_form_neighbor_state: Option<CAState>,
    pub rule_form_operator: Option<RelationalOperator>,
    pub rule_form_threshold: String,
    pub rule_form_next_state: Option<CAState>,
    pub rule_form_error: Option<String>,
    // Grid dimensions input
    pub grid_width_input: String,
    pub grid_height_input: String,

    // For picking next state on canvas click
    pub selected_paint_state_id: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    Definition,
    Simulation,
    // ModelImage, // You mentioned "image of the created model"
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
        let grid = CAGrid::new(DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT);
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
                    // TESTAR
                    // self.grid = step_simulation(&self.grid, &self.rules);
                    // self.grid_cache.clear();
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
                self.grid = CAGrid::new(width, height);
                self.grid_cache.clear();
            }
            Message::ResetGrid => {
                self.grid = CAGrid::new(self.grid.width, self.grid.height);
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
                // TESTAR
                // self.grid = step_simulation(&self.grid, &self.rules);
                // self.grid_cache.clear();
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
