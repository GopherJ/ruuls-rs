#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::ops::{BitAnd, BitOr};

// ***********************************************************************
// STATUS
// **********************************************************************
/// The status of a rule check
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Status {
    /// Rule was satisfied
    Met,
    /// Rule was not satisfied
    NotMet,
    /// There was not enough information to evaluate
    Unknown,
}

impl BitAnd for Status {
    type Output = Status;
    fn bitand(self, rhs: Status) -> Status {
        match (self, rhs) {
            (Status::Met, Status::Met) => Status::Met,
            (Status::NotMet, _) | (_, Status::NotMet) => Status::NotMet,
            (_, _) => Status::Unknown,
        }
    }
}

impl BitOr for Status {
    type Output = Status;
    fn bitor(self, rhs: Status) -> Status {
        match (self, rhs) {
            (Status::NotMet, Status::NotMet) => Status::NotMet,
            (Status::Met, _) | (_, Status::Met) => Status::Met,
            (_, _) => Status::Unknown,
        }
    }
}

// ***********************************************************************
// Rule
// **********************************************************************

/// Representation of a node in the rules tree
///
/// It is unnecessary to interact with this type outside of calling `Rule::check()`,
/// to construct the rules tree use the [convenience functions][1] in the module root.
///
/// [1]: index.html#functions
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Rule {
    And {
        rules: Vec<Rule>,
    },
    Or {
        rules: Vec<Rule>,
    },
    NumberOf {
        n: usize,
        rules: Vec<Rule>,
    },
    Rule {
        field: String,
        #[cfg_attr(feature = "serde", serde(flatten))]
        constraint: Constraint,
    },
}

impl Rule {
    /// Starting at this node, recursively check (depth-first) any child nodes and
    /// aggregate the results
    pub fn check(&self, info: &HashMap<String, String>) -> RuleResult {
        match *self {
            Rule::And { ref rules } => {
                let mut status = Status::Met;
                let children = rules
                    .iter()
                    .map(|c| c.check(info))
                    .inspect(|r| status = status & r.status)
                    .collect::<Vec<_>>();
                RuleResult {
                    name: "And".into(),
                    status,
                    children,
                }
            }
            Rule::Or { ref rules } => {
                let mut status = Status::NotMet;
                let children = rules
                    .iter()
                    .map(|c| c.check(info))
                    .inspect(|r| status = status | r.status)
                    .collect::<Vec<_>>();
                RuleResult {
                    name: "Or".into(),
                    status,
                    children,
                }
            }
            Rule::NumberOf {
                n: count,
                ref rules,
            } => {
                let mut met_count = 0;
                let mut failed_count = 0;
                let children = rules
                    .iter()
                    .map(|c| c.check(info))
                    .inspect(|r| {
                        if r.status == Status::Met {
                            met_count += 1;
                        } else if r.status == Status::NotMet {
                            failed_count += 1;
                        }
                    })
                    .collect::<Vec<_>>();
                let status = if met_count >= count {
                    Status::Met
                } else if failed_count >= children.len() - count + 1 {
                    Status::NotMet
                } else {
                    Status::Unknown
                };
                RuleResult {
                    name: format!("At least {} of", count),
                    status,
                    children,
                }
            }
            Rule::Rule {
                ref field,
                ref constraint,
            } => {
                let status = if let Some(s) = info.get(field) {
                    constraint.check(s)
                } else {
                    Status::Unknown
                };
                RuleResult {
                    name: field.to_owned(),
                    status,
                    children: Vec::new(),
                }
            }
        }
    }
}

// ***********************************************************************
// CONSTRAINT
// **********************************************************************
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all(serialize = "snake_case")))]
#[cfg_attr(feature = "serde", serde(tag = "operator", content = "value"))]
pub enum Constraint {
    StringEquals(String),
    IntEquals(i32),
    IntRange(i32, i32),
    BoolEquals(bool),
}

impl Constraint {
    pub fn check(&self, val: &str) -> Status {
        match *self {
            Constraint::StringEquals(ref s) => {
                if val == s {
                    Status::Met
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntEquals(i) => {
                let parse_res = val.parse::<i32>();
                if let Ok(val) = parse_res {
                    if val == i {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntRange(start, end) => {
                let parse_res = val.parse::<i32>();
                if let Ok(val) = parse_res {
                    if start <= val && val <= end {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::BoolEquals(b) => {
                let bool_val = &val.to_lowercase() == "true";
                if bool_val == b {
                    Status::Met
                } else {
                    Status::NotMet
                }
            }
        }
    }
}

// ***********************************************************************
// Rule RESULT
// **********************************************************************
/// Result of checking a rules tree.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RuleResult {
    /// Human-friendly description of the rule
    pub name: String,
    /// top-level status of this result
    pub status: Status,
    /// Results of any sub-rules
    pub children: Vec<RuleResult>,
}
