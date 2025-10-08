use std::fmt;

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

// Represents a single transition rule
#[derive(Debug, Clone)]
pub struct TransitionRule {
    pub current_state_id: u8,

    pub neighbor_state_id_to_count: Vec<u8>,
    pub operator: Vec<RelationalOperator>,
    pub neighbor_count_threshold: Vec<u8>,
    pub combiner: Vec<ConditionCombiner>,
    pub probability: f32,

    pub next_state_id: u8,
    pub current_state_name: String,
    pub neighbor_state_names: Vec<String>,
    pub next_state_name: String,
}

impl TransitionRule {
    pub fn conditions_as_string(&self) -> String {
        let n = self.neighbor_state_id_to_count.len();
        if n == 0 {
            return "(no conditions)".to_string();
        }

        let mut parts: Vec<String> = Vec::new();

        for i in 0..n {
            let neighbor_name = self
                .neighbor_state_names
                .get(i)
                .cloned()
                .unwrap_or_else(|| format!("State {}", self.neighbor_state_id_to_count[i]));

            let op = self
                .operator
                .get(i)
                .map(|o| o.to_string())
                .unwrap_or("==".to_string());

            let thr = self
                .neighbor_count_threshold
                .get(i)
                .map(|t| t.to_string())
                .unwrap_or("?".to_string());

            let cond = format!("count({}) {} {}", neighbor_name, op, thr);

            if i == 0 {
                parts.push(cond);
            } else {
                let comb = self
                    .combiner
                    .get(i - 1)
                    .map(|c| c.to_string())
                    .unwrap_or("AND".to_string());
                parts.push(format!("{} {}", comb, cond));
            }
        }

        parts.join(" ")
    }
}
