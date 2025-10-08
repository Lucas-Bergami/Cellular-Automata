use crate::messages::Message;
use crate::state::ca_grid::{CAGrid, Neighborhood};
use crate::state::ca_state::CAState;
use crate::state::exemple::ExampleModel;
use crate::state::transition_rule::{ConditionCombiner, RelationalOperator, TransitionRule};
use iced::widget::canvas::Cache;
use iced::widget::{button, column, row, text};
use iced::{executor, theme, Application, Color, Command, Element, Point, Subscription, Theme};
use rand::Rng;
use rayon::prelude::*;
use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

pub struct ConditionForm {
    pub neighbor_state: Option<CAState>,
    pub operator: Option<RelationalOperator>,
    pub threshold: String,
    pub combiner: Option<ConditionCombiner>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    Definition,
    Simulation,
}

const DEFAULT_GRID_WIDTH: usize = 50;
const DEFAULT_GRID_HEIGHT: usize = 40;
const DEFAULT_STATE_ID: u8 = 1;

fn parse_rule(line: &str, states: &[CAState]) -> Result<TransitionRule, String> {
    // println!("\n[DEBUG] Parsing rule line: {}", line);

    let line = line.trim();

    if !line.starts_with("IF current is") {
        return Err("Line does not start with IF current is".into());
    }

    // Localiza a posição do "THEN next is"
    let then_keyword = "THEN next is";
    let then_pos = match line.find(then_keyword) {
        Some(p) => p,
        None => return Err("Missing THEN next is".into()),
    };

    let if_keyword = "IF current is";
    let if_pos = line
        .find(if_keyword)
        .ok_or_else(|| "Missing IF current is".to_string())?;
    let between = line[if_pos + if_keyword.len()..then_pos].trim();
    let then_part = line[then_pos + then_keyword.len()..].trim();

    // println!("[DEBUG] between (IF..THEN) = '{}'", between);
    // println!("[DEBUG] then_part (after THEN) = '{}'", then_part);

    // --- extrai probabilidade (se houver) ---
    let (then_core, probability) = if let Some(with_pos) = then_part.find("WITH PROB") {
        let core = then_part[..with_pos].trim().to_string();

        let prob_str_opt = then_part
            .get(with_pos + 9..) // 9 = tamanho de "WITH PROB"
            .map(|s| s.trim().split_whitespace().next());

        let final_prob = if let Some(Some(p_str)) = prob_str_opt {
            match p_str.parse::<f32>() {
                Ok(p) => {
                    let clamped = p.clamp(0.0, 1.0);
                    if clamped != p {
                        // println!(
                        //     "[WARN] Probability {} out of range [0.0, 1.0], clamped to {}",
                        //     p, clamped
                        // );
                    }
                    clamped
                }
                Err(_) => {
                    // println!(
                    //     "[WARN] Cannot parse probability '{}', defaulting to 1.0",
                    //     p_str
                    // );
                    1.0
                }
            }
        } else {
            // println!("[WARN] Malformed probability format after 'WITH PROB', defaulting to 1.0");
            1.0
        };

        // println!("[DEBUG] probability = {}", final_prob);
        (core, final_prob)
    } else {
        (then_part.to_string(), 1.0)
    };

    // --- extrai next state (entre aspas) ---
    let next_name = if let Some(start) = then_core.find('\'') {
        if let Some(rel_end) = then_core[start + 1..].find('\'') {
            then_core[start + 1..start + 1 + rel_end].trim().to_string()
        } else {
            return Err("Malformed next state (missing closing quote)".into());
        }
    } else {
        return Err("Malformed next state (missing opening quote)".into());
    };

    // println!("[DEBUG] next_name = '{}'", next_name);

    // --- extrai current state ---
    let (current_name, cond_substr) = if let Some(start) = between.find('\'') {
        if let Some(rel_end) = between[start + 1..].find('\'') {
            let name = between[start + 1..start + 1 + rel_end].trim().to_string();
            let after = between[start + 1 + rel_end + 1..].trim();
            (name, after.to_string())
        } else {
            return Err("Malformed current state (missing closing quote)".into());
        }
    } else {
        return Err("Malformed current state (missing opening quote)".into());
    };

    // println!("[DEBUG] current_name = '{}'", current_name);
    // println!("[DEBUG] cond_substr   = '{}'", cond_substr);

    let current_state_id = states
        .iter()
        .find(|s| s.name == current_name)
        .map(|s| s.id)
        .ok_or_else(|| format!("Unknown current state: {}", current_name))?;

    let next_state_id = states
        .iter()
        .find(|s| s.name == next_name)
        .map(|s| s.id)
        .ok_or_else(|| format!("Unknown next state: {}", next_name))?;

    // --- parse conditions (igual ao seu código atual) ---
    let mut neighbor_state_id_to_count: Vec<u8> = Vec::new();
    let mut neighbor_count_threshold: Vec<u8> = Vec::new();
    let mut operator: Vec<RelationalOperator> = Vec::new();
    let mut combiner: Vec<ConditionCombiner> = Vec::new();
    let mut neighbor_state_names: Vec<String> = Vec::new();

    let cond_trimmed = if cond_substr.starts_with("AND") {
        cond_substr[3..].trim().to_string()
    } else {
        cond_substr.trim().to_string()
    };

    if !cond_trimmed.is_empty() && cond_trimmed != "(no conditions)" {
        let tokens: Vec<&str> = cond_trimmed.split_whitespace().collect();
        //println!("[DEBUG] condition tokens = {:?}", tokens);

        let mut i = 0usize;
        while i < tokens.len() {
            let tok = tokens[i];
            if tok.starts_with("count(") {
                let name = tok
                    .trim_start_matches("count(")
                    .trim_end_matches(')')
                    .to_string();
                neighbor_state_names.push(name.clone());

                let neighbor_id = states
                    .iter()
                    .find(|s| s.name == name)
                    .map(|s| s.id)
                    .unwrap_or(0u8);
                neighbor_state_id_to_count.push(neighbor_id);

                if i + 1 < tokens.len() {
                    let op_tok = tokens[i + 1];
                    let op = match op_tok {
                        "==" => RelationalOperator::Equals,
                        "!=" => RelationalOperator::NotEquals,
                        "<" => RelationalOperator::LessThan,
                        "<=" => RelationalOperator::LessOrEqual,
                        ">" => RelationalOperator::GreaterThan,
                        ">=" => RelationalOperator::GreaterOrEqual,
                        _ => RelationalOperator::Equals,
                    };
                    operator.push(op);
                } else {
                    operator.push(RelationalOperator::Equals);
                }

                if i + 2 < tokens.len() {
                    let thr_tok = tokens[i + 2];
                    let thr_clean = thr_tok.trim_end_matches(',').trim();
                    let thr = thr_clean.parse::<u8>().unwrap_or(0u8);
                    neighbor_count_threshold.push(thr);
                } else {
                    neighbor_count_threshold.push(0);
                }

                i += 3;
            } else {
                match tok {
                    "AND" => {
                        combiner.push(ConditionCombiner::And);
                        i += 1;
                    }
                    "OR" => {
                        combiner.push(ConditionCombiner::Or);
                        i += 1;
                    }
                    "XOR" => {
                        combiner.push(ConditionCombiner::Xor);
                        i += 1;
                    }
                    _ => i += 1,
                }
            }
        }
    }

    Ok(TransitionRule {
        current_state_id,
        neighbor_state_id_to_count,
        operator,
        neighbor_count_threshold,
        combiner,
        next_state_id,
        current_state_name: current_name.to_string(),
        neighbor_state_names,
        next_state_name: next_name.to_string(),
        probability,
    })
}

pub struct CASimulator {
    pub fullscreen_mode: bool,
    pub active_tab: TabId,
    pub states: Vec<CAState>,
    pub rules: Vec<TransitionRule>,
    pub grid: CAGrid,
    pub grid_cache: Cache,
    pub simulation_timer: Option<Instant>,
    pub is_simulating: bool,
    pub simulation_speed_ms: u64, // Milliseconds per step
    pub zoom: Cell<f32>,
    pub offset: Cell<Point>,
    pub right_mouse_pressed: Cell<bool>, // panning
    pub last_mouse_pos: RefCell<Option<Point>>,

    // --- UI Input State ---
    // State creation
    pub new_state_name: String,
    pub new_state_color_r: String, // Store as string for input, parse later
    pub new_state_color_g: String,
    pub new_state_color_b: String,

    // Rule creation
    pub rule_form_current_state: Option<CAState>,
    pub rule_form_next_state: Option<CAState>,
    pub rule_form_error: Option<String>,
    pub rule_form_conditions: Vec<ConditionForm>,
    pub rule_form_probability: String,

    // Grid dimensions input
    pub grid_width_input: String,
    pub grid_height_input: String,

    // For picking next state on canvas click
    pub selected_paint_state_id: u8,
    pub mouse_pressed: Cell<bool>,
    pub last_painted_cell: RefCell<Option<(usize, usize)>>,
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
                weight: 5,
            },
            CAState {
                id: 1,
                name: "Alive".to_string(),
                color: Color::new(0.0, 1.0, 0.0, 1.0),
                weight: 5,
            },
        ];
        let grid = CAGrid::new(
            DEFAULT_GRID_WIDTH,
            DEFAULT_GRID_HEIGHT,
            initial_states.clone(),
            Neighborhood::Moore,
        );
        let initial_rules = vec![
            // Alive -> Alive (if neighbors == 2)
            TransitionRule {
                current_state_id: 1,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::Equals],
                neighbor_count_threshold: vec![2],
                combiner: vec![],
                next_state_id: 1,
                current_state_name: "Alive".into(),
                neighbor_state_names: vec!["Alive".into()],
                next_state_name: "Alive".into(),
                probability: 1.0,
            },
            // Alive -> Alive (if neighbors == 3)
            TransitionRule {
                current_state_id: 1,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::Equals],
                neighbor_count_threshold: vec![3],
                combiner: vec![],
                next_state_id: 1,
                current_state_name: "Alive".into(),
                neighbor_state_names: vec!["Alive".into()],
                next_state_name: "Alive".into(),
                probability: 1.0,
            },
            // Dead -> Alive (if neighbors == 3)
            TransitionRule {
                current_state_id: 0,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::Equals],
                neighbor_count_threshold: vec![3],
                combiner: vec![],
                next_state_id: 1,
                current_state_name: "Dead".into(),
                neighbor_state_names: vec!["Alive".into()],
                next_state_name: "Alive".into(),
                probability: 1.0,
            },
            // Alive -> Dead (if neighbors < 2)
            TransitionRule {
                current_state_id: 1,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::LessThan],
                neighbor_count_threshold: vec![2],
                combiner: vec![],
                next_state_id: 0,
                current_state_name: "Alive".into(),
                neighbor_state_names: vec!["Alive".into()],
                next_state_name: "Dead".into(),
                probability: 1.0,
            },
            // Alive -> Dead (if neighbors > 3)
            TransitionRule {
                current_state_id: 1,
                neighbor_state_id_to_count: vec![1],
                operator: vec![RelationalOperator::GreaterThan],
                neighbor_count_threshold: vec![3],
                combiner: vec![],
                next_state_id: 0,
                current_state_name: "Alive".into(),
                neighbor_state_names: vec!["Alive".into()],
                next_state_name: "Dead".into(),
                probability: 1.0,
            },
        ];
        (
            CASimulator {
                fullscreen_mode: false,
                active_tab: TabId::Definition,
                states: initial_states,
                rules: initial_rules,
                grid,
                grid_cache: Cache::new(),
                simulation_timer: None,
                is_simulating: false,
                simulation_speed_ms: 200, // Default speed
                zoom: Cell::new(1.0),
                offset: Cell::new(Point::new(0.0, 0.0)),
                right_mouse_pressed: Cell::new(false),
                last_mouse_pos: RefCell::new(None),

                new_state_name: String::new(),
                new_state_color_r: "0".to_string(),
                new_state_color_g: "0".to_string(),
                new_state_color_b: "0".to_string(),
                rule_form_probability: "1.0".to_string(),

                rule_form_current_state: None,
                rule_form_next_state: None,
                rule_form_error: None,
                rule_form_conditions: vec![],

                grid_width_input: DEFAULT_GRID_WIDTH.to_string(),
                grid_height_input: DEFAULT_GRID_HEIGHT.to_string(),
                selected_paint_state_id: DEFAULT_STATE_ID,
                mouse_pressed: Cell::new(false),
                last_painted_cell: RefCell::new(None),
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
            Message::Tick(()) => {
                if self.is_simulating {
                    self.step_simulation_logic();
                }
            }

            // --- State Definition Messages ---
            Message::StateNameChanged(name) => self.new_state_name = name,
            Message::StateColorRChanged(r) => self.new_state_color_r = r,
            Message::StateColorGChanged(g) => self.new_state_color_g = g,
            Message::StateColorBChanged(b) => self.new_state_color_b = b,
            Message::RuleProbabilityChanged(val) => {
                self.rule_form_probability = val;
            }
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
                        weight: 1,
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
                                weight: 5,
                            },
                            CAState {
                                id: 1,
                                name: "Alive".into(),
                                color: Color::from_rgb8(0, 255, 0),
                                weight: 5,
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
                                neighbor_state_names: vec!["Alive".into()],
                                next_state_name: "Alive".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 1,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::Equals],
                                neighbor_count_threshold: vec![3],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Alive".into(),
                                neighbor_state_names: vec!["Alive".into()],
                                next_state_name: "Alive".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 0,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::Equals],
                                neighbor_count_threshold: vec![3],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Dead".into(),
                                neighbor_state_names: vec!["Alive".into()],
                                next_state_name: "Alive".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 1,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::LessThan],
                                neighbor_count_threshold: vec![2],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Alive".into(),
                                neighbor_state_names: vec!["Alive".into()],
                                next_state_name: "Dead".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 1,
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::GreaterThan],
                                neighbor_count_threshold: vec![3],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Alive".into(),
                                neighbor_state_names: vec!["Alive".into()],
                                next_state_name: "Dead".into(),
                                probability: 1.0,
                            },
                        ];
                    }

                    ExampleModel::Wireworld => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Empty".into(),
                                color: Color::BLACK,
                                weight: 10,
                            },
                            CAState {
                                id: 1,
                                name: "ElectronHead".into(),
                                color: Color::from_rgb8(0, 0, 255),
                                weight: 0,
                            },
                            CAState {
                                id: 2,
                                name: "ElectronTail".into(),
                                color: Color::from_rgb8(255, 0, 0),
                                weight: 0,
                            },
                            CAState {
                                id: 3,
                                name: "Conductor".into(),
                                color: Color::from_rgb8(255, 255, 0),
                                weight: 0,
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
                                current_state_name: "ElectronHead".into(),
                                neighbor_state_names: vec![],
                                next_state_name: "ElectronTail".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 2, // Tail -> Conductor
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 3,
                                current_state_name: "ElectronTail".into(),
                                neighbor_state_names: vec![],
                                next_state_name: "Conductor".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 3, // Conductor -> Head if 1 or 2 neighbors are Head
                                neighbor_state_id_to_count: vec![1, 1],
                                operator: vec![
                                    RelationalOperator::Equals,
                                    RelationalOperator::Equals,
                                ],
                                neighbor_count_threshold: vec![1, 2],
                                combiner: vec![ConditionCombiner::Or],
                                next_state_id: 1,
                                current_state_name: "Conductor".into(),
                                neighbor_state_names: vec![
                                    "ElectronHead".into(),
                                    "ElectronHead".into(),
                                ],
                                next_state_name: "ElectronHead".into(),
                                probability: 1.0,
                            },
                        ];
                    }

                    ExampleModel::Greenberg => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Off".into(),
                                color: Color::BLACK,
                                weight: 10,
                            },
                            CAState {
                                id: 1,
                                name: "On".into(),
                                color: Color::from_rgb8(0, 0, 255),
                                weight: 10,
                            },
                            CAState {
                                id: 2,
                                name: "Dying".into(),
                                color: Color::from_rgb8(255, 0, 0),
                                weight: 10,
                            },
                        ];

                        self.rules = vec![
                            TransitionRule {
                                current_state_id: 0, // Off -> On if 2 neighbors are On
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::Equals],
                                neighbor_count_threshold: vec![2],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Off".into(),
                                neighbor_state_names: vec!["On".into()],
                                next_state_name: "On".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 1, // On -> Dying
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 2,
                                current_state_name: "On".into(),
                                neighbor_state_names: vec![],
                                next_state_name: "Dying".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 2, // Dying -> Off
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Dying".into(),
                                neighbor_state_names: vec![],
                                next_state_name: "Off".into(),
                                probability: 1.0,
                            },
                        ];
                    }

                    ExampleModel::TuringPatterns => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Empty".into(),
                                color: Color::BLACK,
                                weight: 10,
                            },
                            CAState {
                                id: 1,
                                name: "Activator".into(),
                                color: Color::from_rgb8(0, 200, 255),
                                weight: 5,
                            },
                            CAState {
                                id: 2,
                                name: "Inhibitor".into(),
                                color: Color::from_rgb8(255, 100, 0),
                                weight: 5,
                            },
                        ];

                        self.rules = vec![
                            TransitionRule {
                                current_state_id: 0, // Empty -> Activator if >=2 neighbors Activator
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::GreaterOrEqual],
                                neighbor_count_threshold: vec![2],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Empty".into(),
                                neighbor_state_names: vec!["Activator".into()],
                                next_state_name: "Activator".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 1, // Activator -> Inhibitor if >=3 neighbors Activator
                                neighbor_state_id_to_count: vec![1],
                                operator: vec![RelationalOperator::GreaterOrEqual],
                                neighbor_count_threshold: vec![3],
                                combiner: vec![],
                                next_state_id: 2,
                                current_state_name: "Activator".into(),
                                neighbor_state_names: vec!["Activator".into()],
                                next_state_name: "Inhibitor".into(),
                                probability: 1.0,
                            },
                            TransitionRule {
                                current_state_id: 2, // Inhibitor -> Empty
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 0,
                                current_state_name: "Inhibitor".into(),
                                neighbor_state_names: vec![],
                                next_state_name: "Empty".into(),
                                probability: 1.0,
                            },
                        ];
                    }

                    ExampleModel::ForestFire => {
                        self.states = vec![
                            CAState {
                                id: 0,
                                name: "Empty".into(),
                                color: Color::BLACK,
                                weight: 10,
                            },
                            CAState {
                                id: 1,
                                name: "Tree".into(),
                                color: Color::from_rgb8(0, 200, 0),
                                weight: 7,
                            },
                            CAState {
                                id: 2,
                                name: "Burning".into(),
                                color: Color::from_rgb8(255, 0, 0),
                                weight: 3,
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
                                neighbor_state_names: vec![],
                                next_state_name: "Empty".into(),
                                probability: 0.8,
                            },
                            TransitionRule {
                                current_state_id: 1, // Tree -> Burning if >=1 neighbor Burning
                                neighbor_state_id_to_count: vec![2],
                                operator: vec![RelationalOperator::GreaterOrEqual],
                                neighbor_count_threshold: vec![1],
                                combiner: vec![],
                                next_state_id: 2,
                                current_state_name: "Tree".into(),
                                neighbor_state_names: vec!["Burning".into()],
                                next_state_name: "Burning".into(),
                                probability: 0.5,
                            },
                            TransitionRule {
                                current_state_id: 0, // Empty -> Tree (budding)
                                neighbor_state_id_to_count: vec![],
                                operator: vec![],
                                neighbor_count_threshold: vec![],
                                combiner: vec![],
                                next_state_id: 1,
                                current_state_name: "Empty".into(),
                                neighbor_state_names: vec![],
                                next_state_name: "Tree".into(),
                                probability: 0.3,
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
                        weight: 1,
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
                        weight: 1,
                    }
                };

                let mut neighbor_ids: Vec<u8> = Vec::new();
                let mut operators: Vec<RelationalOperator> = Vec::new();
                let mut thresholds: Vec<u8> = Vec::new();
                let mut combiners: Vec<ConditionCombiner> = Vec::new();

                for (idx, cond) in self.rule_form_conditions.iter().enumerate() {
                    if let Some(state) = &cond.neighbor_state {
                        neighbor_ids.push(state.id);
                    } else {
                        errors.push(format!(
                            "Neighbor State não selecionado na condição {}",
                            idx + 1
                        ));
                        neighbor_ids.push(0);
                    }

                    if let Some(op) = cond.operator {
                        operators.push(op);
                    } else {
                        errors.push(format!("Operador não selecionado na condição {}", idx + 1));
                        operators.push(RelationalOperator::Equals);
                    }

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
                    if neighbor_ids.is_empty() {
                        String::new()
                    } else {
                        let mut names: Vec<String> = Vec::with_capacity(neighbor_ids.len());
                        for id in &neighbor_ids {
                            if let Some(s) = self.states.iter().find(|st| st.id == *id) {
                                names.push(s.name.clone());
                            } else {
                                names.push(format!("State {}", id));
                            }
                        }
                        names.join(",")
                    };

                    let probability: f32 = match self.rule_form_probability.parse::<f32>() {
                        Ok(p) if (0.0..=1.0).contains(&p) => p,
                        _ => {
                            errors
                                .push("Probabilidade inválida (use valor entre 0.0 e 1.0)".into());
                            1.0
                        }
                    };

                    self.rules.push(TransitionRule {
                        current_state_id: cur.id,
                        neighbor_state_id_to_count: neighbor_ids,
                        operator: operators,
                        neighbor_count_threshold: thresholds,
                        combiner: combiners,
                        next_state_id: nxt.id,
                        current_state_name: cur.name.clone(),
                        neighbor_state_names: self
                            .rule_form_conditions
                            .iter()
                            .map(|c| {
                                c.neighbor_state
                                    .as_ref()
                                    .map_or("".into(), |s| s.name.clone())
                            })
                            .collect(),
                        next_state_name: nxt.name.clone(),
                        probability,
                    });

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
            Message::StateWeightChanged(idx, val) => {
                if let Some(state) = self.states.get_mut(idx) {
                    if val.trim().is_empty() {
                        state.weight = 0;
                    } else if let Ok(parsed) = val.parse::<u8>() {
                        state.weight = parsed;
                    }
                }
            }
            Message::ExportRules => {
                use std::fs::File;
                use std::io::Write;

                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Salvar regras")
                    .add_filter("Arquivo de texto", &["txt"])
                    .save_file()
                {
                    if let Ok(mut file) = File::create(&path) {
                        writeln!(
                            file,
                            "WIDTH {} HEIGHT {}",
                            self.grid.width, self.grid.height
                        )
                        .ok();

                        writeln!(file, "STATE {{").ok();
                        for state in &self.states {
                            let r = (state.color.r * 255.0).round() as u8;
                            let g = (state.color.g * 255.0).round() as u8;
                            let b = (state.color.b * 255.0).round() as u8;
                            let w = state.weight;
                            writeln!(file, "    {}({}, {}, {}, {})", state.name, r, g, b, w).ok();
                        }
                        writeln!(file, "}}\n").ok();

                        writeln!(file, "RULES {{").ok();
                        for rule in &self.rules {
                            let conditions = rule.conditions_as_string();
                            writeln!(
                                file,
                                "    IF current is '{}' AND {} THEN next is '{}' WITH PROB {}",
                                rule.current_state_name,
                                conditions,
                                rule.next_state_name,
                                rule.probability
                            )
                            .ok();
                        }
                        writeln!(file, "}}").ok();

                        println!("Rules, states and probabilities exported to {:?}", path);
                    } else {
                        println!("Error creating file: {:?}", path);
                    }
                } else {
                    println!("Export canceled by user");
                }

                return Command::none();
            }

            Message::ImportRules => {
                use std::fs::File;
                use std::io::{BufRead, BufReader};

                let path_opt = rfd::FileDialog::new()
                    .add_filter("Text Files", &["txt"])
                    .pick_file();

                if let Some(path) = path_opt {
                    if let Ok(file) = File::open(&path) {
                        let reader = BufReader::new(file);

                        self.states.clear();
                        self.rules.clear();

                        let mut grid_width = 0;
                        let mut grid_height = 0;

                        let mut in_states = false;
                        let mut in_rules = false;

                        for line in reader.lines().flatten() {
                            let line = line.trim();

                            if line.is_empty() {
                                continue;
                            }

                            if line.starts_with("WIDTH") {
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 4 {
                                    grid_width = parts[1].parse::<usize>().unwrap_or(50);
                                    grid_height = parts[3].parse::<usize>().unwrap_or(50);
                                }
                            } else if line.starts_with("STATE") && line.contains('{') {
                                in_states = true;
                                in_rules = false;
                            } else if line.starts_with("RULES") && line.contains('{') {
                                in_rules = true;
                                in_states = false;
                            } else if line == "}" {
                                in_states = false;
                                in_rules = false;
                            } else if in_states {
                                // Parse de estado: nome(r,g,b,weight)
                                if let Some(start) = line.find('(') {
                                    if let Some(end) = line.find(')') {
                                        let name =
                                            line[..start].trim().trim_end_matches(',').to_string();
                                        let nums: Vec<u8> = line[start + 1..end]
                                            .split(',')
                                            .map(|v| v.trim().parse().unwrap_or(0))
                                            .collect();

                                        let (r, g, b, weight) = if nums.len() == 4 {
                                            (nums[0], nums[1], nums[2], nums[3])
                                        } else if nums.len() == 3 {
                                            (nums[0], nums[1], nums[2], 1)
                                        } else {
                                            (0, 0, 0, 1)
                                        };

                                        let color = Color::from_rgb8(r, g, b);
                                        let id = self.states.len() as u8;

                                        self.states.push(CAState {
                                            id,
                                            name,
                                            color,
                                            weight,
                                        });
                                    }
                                }
                            } else if in_rules {
                                if let Ok(rule) = parse_rule(line, &self.states) {
                                    self.rules.push(rule);
                                }
                            }
                        }

                        self.grid.width = grid_width;
                        self.grid.height = grid_height;

                        self.grid_cache.clear();

                        println!("Imported rules, states and grid size from {:?}", path);
                    } else {
                        println!("Error opening file: {:?}", path);
                    }
                } else {
                    println!("No file selected.");
                }

                return Command::none();
            }
            Message::ToggleFullscreen => {
                self.fullscreen_mode = !self.fullscreen_mode;
            }
            Message::SaveGrid => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name("grid.json")
                    .save_file()
                {
                    if let Ok(json) = serde_json::to_string(&self.grid) {
                        if let Err(e) = std::fs::write(&path, json) {
                            eprintln!("Failed to save grid: {}", e);
                        }
                    } else {
                        eprintln!("Failed to serialize grid");
                    }
                }
            }
            Message::LoadGrid => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file()
                {
                    match std::fs::read_to_string(&path) {
                        Ok(data) => match serde_json::from_str::<CAGrid>(&data) {
                            Ok(grid) => {
                                self.grid = grid;
                            }
                            Err(e) => eprintln!("Failed to parse grid JSON: {}", e),
                        },
                        Err(e) => eprintln!("Failed to read file: {}", e),
                    }
                }
            }
            Message::NeighborhoodChanged(nb) => self.grid.neighborhood = nb,
            Message::GridWidthChanged(w) => self.grid_width_input = w,
            Message::GridHeightChanged(h) => self.grid_height_input = h,
            Message::ApplyGridSize => {
                let width = self.grid_width_input.parse().unwrap_or(DEFAULT_GRID_WIDTH);
                let height = self
                    .grid_height_input
                    .parse()
                    .unwrap_or(DEFAULT_GRID_HEIGHT);
                self.grid = CAGrid::new(width, height, self.states.clone(), self.grid.neighborhood);
                self.grid_cache.clear();
            }
            Message::ResetGrid => {
                self.grid = CAGrid::new(
                    self.grid.width,
                    self.grid.height,
                    self.states.clone(),
                    self.grid.neighborhood,
                );
                self.grid_cache.clear();
                self.zoom.set(1.0);
                self.offset = Point::new(0.0, 0.0).into();
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

    fn view(&self) -> Element<'_, Message> {
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
        ]
        .spacing(10);

        let content = match self.active_tab {
            TabId::Definition => self.view_definition_tab(),
            TabId::Simulation => self.view_simulation_tab(),
        };

        column![header, tab_buttons, content]
            .spacing(20)
            .padding(20)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.is_simulating {
            iced::time::every(Duration::from_millis(self.simulation_speed_ms))
                .map(|_| Message::Tick(()))
        } else {
            Subscription::none()
        }
    }
}

impl CASimulator {
    fn step_simulation_logic(&mut self) {
        if self.states.is_empty() {
            return;
        }

        let width = self.grid.width;
        let height = self.grid.height;
        let grid_size = width * height;

        let current_grid_flat: Vec<u8> = self
            .grid
            .cells
            .iter()
            .flat_map(|row| row.iter())
            .copied()
            .collect();
        let mut next_grid_flat = vec![0u8; grid_size];

        let mut neighbor_counts: Vec<Vec<u8>> = vec![vec![0; grid_size]; self.states.len()];
        for state in &self.states {
            let id = state.id as usize;
            for r in 0..height {
                for c in 0..width {
                    neighbor_counts[id][r * width + c] = self.grid.count_neighbors(r, c, state.id);
                }
            }
        }

        let threshold = 10_000;

        if grid_size >= threshold {
            next_grid_flat
                .par_iter_mut()
                .enumerate()
                .for_each(|(idx, cell)| {
                    let current_cell_state_id = current_grid_flat[idx];
                    let mut new_state_id = current_cell_state_id;

                    let mut rng = rand::rng();

                    for rule in &self.rules {
                        if rule.current_state_id != current_cell_state_id {
                            continue;
                        }

                        if rng.random::<f32>() > rule.probability {
                            continue;
                        }

                        let final_result = if rule.neighbor_state_id_to_count.is_empty() {
                            true
                        } else {
                            let mut res = true;
                            for i in 0..rule.neighbor_state_id_to_count.len() {
                                let neighbor_state = rule.neighbor_state_id_to_count[i] as usize;
                                let op = rule.operator[i];
                                let thr = rule.neighbor_count_threshold[i];

                                let neighbor_count = neighbor_counts[neighbor_state][idx];
                                let condition = op.evaluate(neighbor_count, thr);

                                if i == 0 {
                                    res = condition;
                                } else {
                                    match rule.combiner[i - 1] {
                                        ConditionCombiner::And => res &= condition,
                                        ConditionCombiner::Or => res |= condition,
                                        ConditionCombiner::Xor => res ^= condition,
                                    }
                                }
                            }
                            res
                        };

                        if final_result {
                            new_state_id = rule.next_state_id;
                            break;
                        }
                    }

                    *cell = new_state_id;
                });
        } else {
            for idx in 0..grid_size {
                let current_cell_state_id = current_grid_flat[idx];
                let mut new_state_id = current_cell_state_id;

                let mut rng = rand::rng();

                for rule in &self.rules {
                    if rule.current_state_id != current_cell_state_id {
                        continue;
                    }

                    if rng.random::<f32>() > rule.probability {
                        continue;
                    }

                    let final_result = if rule.neighbor_state_id_to_count.is_empty() {
                        true
                    } else {
                        let mut res = true;
                        for i in 0..rule.neighbor_state_id_to_count.len() {
                            let neighbor_state = rule.neighbor_state_id_to_count[i] as usize;
                            let op = rule.operator[i];
                            let thr = rule.neighbor_count_threshold[i];

                            let neighbor_count = neighbor_counts[neighbor_state][idx];
                            let condition = op.evaluate(neighbor_count, thr);

                            if i == 0 {
                                res = condition;
                            } else {
                                match rule.combiner[i - 1] {
                                    ConditionCombiner::And => res &= condition,
                                    ConditionCombiner::Or => res |= condition,
                                    ConditionCombiner::Xor => res ^= condition,
                                }
                            }
                        }
                        res
                    };

                    if final_result {
                        new_state_id = rule.next_state_id;
                        break;
                    }
                }

                next_grid_flat[idx] = new_state_id;
            }
        }

        for r in 0..height {
            for c in 0..width {
                self.grid.cells[r][c] = next_grid_flat[r * width + c];
            }
        }

        self.grid_cache.clear();
    }
}
