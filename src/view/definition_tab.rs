use crate::controller::{app::CASimulator, message::Message};
use crate::model::ca::RelationalOperator;
use iced::{
    theme,
    widget::{button, column, row, text, text_input, Column, Container, PickList, Scrollable},
    Alignment, Element, Length,
};

impl CASimulator {
    pub fn view_definition_tab(&self) -> Element<Message> {
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
}
