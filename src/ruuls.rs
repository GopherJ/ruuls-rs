#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    ops::{BitAnd, BitOr, Not},
};

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

impl Not for Status {
    type Output = Status;

    fn not(self) -> Self::Output {
        match self {
            Status::Met => Status::NotMet,
            Status::NotMet => Status::Met,
            Status::Unknown => Status::Unknown,
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
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum Rule {
    And {
        and: Vec<Rule>,
    },
    Or {
        or: Vec<Rule>,
    },
    AtLeast {
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
    pub fn check_map(&self, info: &HashMap<String, String>) -> RuleResult {
        match *self {
            Rule::And { ref and } => {
                let mut status = Status::Met;
                let children = and
                    .iter()
                    .map(|c| c.check_map(info))
                    .inspect(|r| status = status & r.status)
                    .collect::<Vec<_>>();
                RuleResult {
                    name: "And".into(),
                    status,
                    children,
                }
            }
            Rule::Or { ref or } => {
                let mut status = Status::NotMet;
                let children = or
                    .iter()
                    .map(|c| c.check_map(info))
                    .inspect(|r| status = status | r.status)
                    .collect::<Vec<_>>();
                RuleResult {
                    name: "Or".into(),
                    status,
                    children,
                }
            }
            Rule::AtLeast {
                n: count,
                ref rules,
            } => {
                let mut met_count = 0;
                let children = rules
                    .iter()
                    .map(|c| c.check_map(info))
                    .inspect(|r| {
                        if r.status == Status::Met {
                            met_count += 1;
                        }
                    })
                    .collect::<Vec<_>>();
                let status = if met_count >= count {
                    Status::Met
                } else {
                    Status::NotMet
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
#[cfg_attr(feature = "serde", serde(rename_all(serialize = "camelCase")))]
#[cfg_attr(feature = "serde", serde(tag = "operator", content = "value"))]
pub enum Constraint {
    StringEquals(String),
    StringNotEquals(String),
    StringIn(Vec<String>),
    StringNotIn(Vec<String>),
    IntEquals(i32),
    IntNotEquals(i32),
    IntIn(Vec<i32>),
    IntNotIn(Vec<i32>),
    IntInRange(i32, i32),
    IntNotInRange(i32, i32),
    LessThan(i32),
    LessThanInclusive(i32),
    GreaterThan(i32),
    GreaterThanInclusive(i32),
    BoolEquals(bool),
}

impl Constraint {
    pub fn check(&self, v: &str) -> Status {
        match *self {
            Constraint::StringEquals(ref s) => {
                if v == s {
                    Status::Met
                } else {
                    Status::NotMet
                }
            }
            Constraint::StringNotEquals(ref s) => {
                if v != s {
                    Status::Met
                } else {
                    Status::NotMet
                }
            }
            Constraint::StringIn(ref ss) => {
                if ss.iter().any(|s| s == v) {
                    Status::Met
                } else {
                    Status::NotMet
                }
            }
            Constraint::StringNotIn(ref ss) => {
                if ss.iter().all(|s| s != v) {
                    Status::Met
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntEquals(num) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if val == num {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntIn(ref nums) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if nums.iter().any(|&num| num == val) {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntNotIn(ref nums) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if nums.iter().all(|&num| num != val) {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntNotEquals(num) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if val != num {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::IntInRange(start, end) => {
                let parse_res = v.parse::<i32>();
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
            Constraint::IntNotInRange(start, end) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if start <= val && val <= end {
                        Status::NotMet
                    } else {
                        Status::Met
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::LessThan(num) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if val < num {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::LessThanInclusive(num) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if val <= num {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::GreaterThan(num) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if val > num {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::GreaterThanInclusive(num) => {
                let parse_res = v.parse::<i32>();
                if let Ok(val) = parse_res {
                    if val >= num {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
                } else {
                    Status::NotMet
                }
            }
            Constraint::BoolEquals(b) => {
                let parse_res = v.parse::<bool>();
                if let Ok(val) = parse_res {
                    if val == b {
                        Status::Met
                    } else {
                        Status::NotMet
                    }
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
