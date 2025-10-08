use crate::app::CASimulator;
use crate::messages::Message;
use crate::state::ca_grid::Neighborhood;
use crate::state::exemple::ExampleModel;
use crate::state::transition_rule::{ConditionCombiner, RelationalOperator};
use iced::widget::{
    button, column, row, text, text_input, Canvas, Column, Container, PickList, Scrollable, Slider,
    Space,
};
use iced::{theme, Alignment, Color, Element, Length};

impl CASimulator {
    pub fn view_definition_tab(&self) -> Element<'_, Message> {
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

        let states_list = if self.states.is_empty() {
            Column::new()
                .push(text("No states defined yet"))
                .spacing(10)
                .width(Length::Fill)
        } else {
            let mut column = Column::new().spacing(10).width(Length::Fill);
            for (idx, state) in self.states.iter().enumerate() {
                column = column.push(
                    row![
                        // Nome
                        text(&state.name).width(Length::Fixed(120.0)),
                        // Cor
                        text(format!(
                            "RGB: ({}, {}, {})",
                            (state.color.r * 255.0) as u8,
                            (state.color.g * 255.0) as u8,
                            (state.color.b * 255.0) as u8
                        ))
                        .width(Length::Fixed(150.0)),
                        // Peso
                        text("Weight:").width(Length::Fixed(60.0)),
                        text_input("Weight", &state.weight.to_string())
                            .on_input(move |val| Message::StateWeightChanged(idx, val))
                            .padding(5)
                            .width(Length::Fixed(80.0)),
                        // Remover
                        button("Remove")
                            .on_press(Message::RemoveState(idx))
                            .style(theme::Button::Destructive)
                            .padding(5),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                );
            }
            column
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
            text("AND the following conditions are met:"),
        ];

        for idx in 0..self.rule_form_conditions.len() {
            let cond = &self.rule_form_conditions[idx];

            let mut condition_row = row![
                PickList::new(
                    available_states_for_picklist.clone(),
                    cond.neighbor_state.clone(),
                    move |s| Message::RuleNeighborStateSelected(idx, s)
                )
                .placeholder("Neighbor State"),
                PickList::new(RelationalOperator::ALL.to_vec(), cond.operator, move |op| {
                    Message::RuleOperatorSelected(idx, op)
                })
                .placeholder("Operator"),
                text_input("Count (e.g., 3)", &cond.threshold)
                    .on_input(move |val| Message::RuleThresholdChanged(idx, val))
                    .padding(5)
                    .width(Length::Fixed(80.0)),
                button("-").on_press(Message::RemoveCondition(idx))
            ]
            .spacing(5);

            if idx < self.rule_form_conditions.len() - 1 {
                condition_row = condition_row.push(
                    PickList::new(
                        ConditionCombiner::ALL.to_vec(),
                        cond.combiner.clone(),
                        move |comb| Message::RuleCombinerSelected(idx, comb),
                    )
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

        rule_creation_panel = rule_creation_panel.push(text("THEN Next State is:")).push(
            PickList::new(
                available_states_for_picklist.clone(),
                self.rule_form_next_state.clone(),
                Message::RuleNextStateSelected,
            )
            .placeholder("Select Next State"),
        );

        rule_creation_panel = rule_creation_panel
            .push(text("Probability (0.0 - 1.0):"))
            .push(
                text_input("e.g., 0.8", &self.rule_form_probability)
                    .on_input(Message::RuleProbabilityChanged)
                    .padding(5)
                    .width(Length::Fixed(100.0)),
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
                                    "IF current is '{}' AND {} THEN next is '{}' WITH PROB '{}'",
                                    rule.current_state_name,
                                    rule.conditions_as_string(),
                                    rule.next_state_name,
                                    rule.probability
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

        let export_import_row = row![
            button("Export Rules").on_press(Message::ExportRules),
            button("Import Rules").on_press(Message::ImportRules),
        ]
        .spacing(10)
        .align_items(Alignment::Center);

        let rules_panel = column![
            text("Defined Rules").size(20),
            Scrollable::new(rules_list)
                .height(Length::Fixed(200.0))
                .width(Length::Fill),
            export_import_row,
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

    pub fn view_simulation_tab(&self) -> Element<'_, Message> {
        if self.fullscreen_mode {
            let controls = row![
                button(if self.is_simulating { "Pause" } else { "Start" })
                    .on_press(Message::ToggleSimulation)
                    .padding(5),
                button("Next Step").on_press(Message::NextStep).padding(5),
                button("Reset Grid").on_press(Message::ResetGrid).padding(5),
                button("Exit Fullscreen")
                    .on_press(Message::ToggleFullscreen)
                    .padding(5),
            ]
            .spacing(10)
            .align_items(Alignment::Center);

            column![
                controls,
                Canvas::new(self).width(Length::Fill).height(Length::Fill)
            ]
            .spacing(20)
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        } else {
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
                    button("Save Grid").on_press(Message::SaveGrid).padding(5),
                    button("Load Grid").on_press(Message::LoadGrid).padding(5),
                    button("Fullscreen")
                        .on_press(Message::ToggleFullscreen)
                        .padding(5),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
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
                    text("Speed (Slow -> Fast):"),
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
            .width(Length::Fill);

            let canvas = Canvas::new(self)
                .width(Length::Fixed(600.0))
                .height(Length::Fixed(600.0));

            Scrollable::new(
                column![
                    controls,
                    row![
                        Space::with_width(Length::Fill),
                        canvas,
                        Space::with_width(Length::Fill),
                    ]
                    .width(Length::Fill)
                ]
                .spacing(20)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }
    }
}
