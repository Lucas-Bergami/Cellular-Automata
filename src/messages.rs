use crate::app::simulator::TabId;
use crate::state::ca_grid::Neighborhood;
use crate::state::exemple::ExampleModel;
use crate::state::transition_rule::ConditionCombiner;
use crate::state::transition_rule::RelationalOperator;
use crate::state::CAState;
#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(TabId),
    Tick(()),

    // State definition
    RuleProbabilityChanged(String),
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
    StateWeightChanged(usize, String),
    ExportRules,
    ImportRules,

    // Grid/Simulation
    ToggleFullscreen,
    SaveGrid,
    LoadGrid,
    NeighborhoodChanged(Neighborhood),
    GridWidthChanged(String),
    GridHeightChanged(String),
    ApplyGridSize,
    ResetGrid,
    ToggleSimulation,
    NextStep,
    SimulationSpeedChanged(f32), // From slider (0-100), map to ms
    PaintStateSelected(CAState), // For selecting which state to paint on click
    PaintCell(usize, usize, u8),
}
