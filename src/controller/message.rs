use crate::controller::app::TabId;
use crate::model::ca::{CAState, RelationalOperator}; // ou ajuste para onde estiver
use iced::widget::canvas;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(TabId),
    Tick(Instant),

    // State definition
    StateNameChanged(String),
    StateColorRChanged(String),
    StateColorGChanged(String),
    StateColorBChanged(String),
    AddState,
    RemoveState(usize),

    // Rule definition
    RuleCurrentStateSelected(CAState),
    RuleNeighborStateSelected(CAState),
    RuleOperatorSelected(RelationalOperator),
    RuleThresholdChanged(String),
    RuleNextStateSelected(CAState),
    AddRule,
    RemoveRule(usize),

    // Grid/Simulation
    GridWidthChanged(String),
    GridHeightChanged(String),
    ApplyGridSize,
    ResetGrid,
    ToggleSimulation,
    NextStep,
    SimulationSpeedChanged(f32),
    CanvasEvent(canvas::Event),
    PaintStateSelected(CAState),
    PaintCell(usize, usize, u8),
}
