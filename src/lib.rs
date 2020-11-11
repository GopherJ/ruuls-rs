//! Simple rules engine that represents requirements as a tree, with each node having one or more requirements in order to be "Met".
//!
//! A tree of rules is constructed, and then the [`.check()`][1] method is called.
//! `map` is a `field: value` mapping of facts that will be given to each node in the tree for testing.
//!
//! Status output can be either `Met`, `NotMet`, or `Unknown` if the tested field is not present in the map.
//!
//! To construct a tree, see the following methods.
//!
//! ## Example
//!
//! ```rust
//! use std::collections::HashMap;
//! extern crate ruuls;
//!
//! let tree = ruuls::and(vec![
//!     ruuls::string_equals("name", "John Doe"),
//!     ruuls::or(vec![
//!         ruuls::int_equals("fav_number", 5),
//!         ruuls::int_in_range("thinking_of", 5, 10)
//!     ])
//! ]);
//! let mut facts = HashMap::new();
//! facts.insert("name".into(), "John Doe".into());
//! facts.insert("fav_number".into(), "5".into());
//! let result = tree.check_map(&facts);
//! println!("{:?}", result);
//! assert!(result.status == ruuls::Status::Met);
//! // result = RuleResult { name: "And", status: Met, children: [RuleResult { name: "Name is John Doe", status: Met, children: [] }, RuleResult { name: "Or", status: Met, children: [RuleResult { name: "Favorite number is 5", status: Met, children: [] }, RuleResult { name: "Thinking of a number between 5 and 10", status: Unknown, children: [] }] }] }
//! ```
//!
//! This creates a tree like the following:
//!
//! ```text
//!                              +---------+
//!                              |   AND   |
//!                              +---------+
//!           _____________________/\_______________
//!          |                                      |
//!          V                                      V
//! +-------------------+                       +--------+
//! | Name is John Doe  |                       |   OR   |
//! +-------------------+                       +--------+
//! | field: "name"     |             ______________/\___________
//! | value: "John Doe" |            |                           |
//! +-------------------+            V                           V
//!                       +----------------------+  +-------------------------+
//!                       | Favorite number is 5 |  | Number between 5 and 10 |
//!                       +----------------------+  +-------------------------+
//!                       | field: "fav_number"  |  | field: "thinking_of"    |
//!                       | value: 5             |  | start: 5                |
//!                       +----------------------+  | end: 10                 |
//!                                                 +-------------------------+
//! ```
//!
//! [1]: enum.Rule.html#method.check

#[cfg(feature = "serde")]
mod ruuls;

pub use crate::ruuls::{Constraint, Rule, RuleResult, Status};

/// Creates a `Rule` where all child `Rule`s must be `Met`
///
/// * If any are `NotMet`, the result will be `NotMet`
/// * If the results contain only `Met` and `Unknown`, the result will be `Unknown`
/// * Only results in `Met` if all children are `Met`
pub fn and(and: Vec<Rule>) -> Rule {
    Rule::And { and }
}

/// Creates a `Rule` where any child `Rule` must be `Met`
///
/// * If any are `Met`, the result will be `Met`
/// * If the results contain only `NotMet` and `Unknown`, the result will be `Unknown`
/// * Only results in `NotMet` if all children are `NotMet`
pub fn or(or: Vec<Rule>) -> Rule {
    Rule::Or { or }
}

/// Creates a `Rule` where `n` child `Rule`s must be `Met`
///
/// * If `>= n` are `Met`, the result will be `Met`, otherwise it'll be `NotMet`
pub fn at_least(n: usize, rules: Vec<Rule>) -> Rule {
    Rule::AtLeast { n, rules }
}

/// Creates a rule for string comparison
pub fn string_equals(field: &str, val: &str) -> Rule {
    Rule::Rule {
        field: field.into(),
        constraint: Constraint::StringEquals(val.into()),
    }
}

/// Creates a rule for int comparison.
///
///If the checked value is not convertible to an integer, the result is `NotMet`
pub fn int_equals(field: &str, val: i64) -> Rule {
    Rule::Rule {
        field: field.into(),
        constraint: Constraint::IntEquals(val),
    }
}

/// Creates a rule for int range comparison with the interval `[start, end]`.
///
/// If the checked value is not convertible to an integer, the result is `NotMet`
pub fn int_in_range(field: &str, start: i64, end: i64) -> Rule {
    Rule::Rule {
        field: field.into(),
        constraint: Constraint::IntInRange(start, end),
    }
}

/// Creates a rule for boolean comparison.
///
/// Only input values of `"true"` (case-insensitive) are considered `true`, all others are considered `false`
pub fn bool_equals(field: &str, val: bool) -> Rule {
    Rule::Rule {
        field: field.into(),
        constraint: Constraint::BoolEquals(val),
    }
}

#[cfg(test)]
mod tests {
    use super::{and, at_least, bool_equals, int_equals, int_in_range, or, string_equals, Status};
    use std::collections::HashMap;

    fn get_test_data() -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("foo".into(), "1".into());
        map.insert("bar".into(), "bar".into());
        map.insert("baz".into(), "true".into());
        map
    }

    #[test]
    fn and_rules() {
        let map = get_test_data();
        // Met & Met == Met
        let mut root = and(vec![int_equals("foo", 1), string_equals("bar", "bar")]);
        let mut res = root.check_map(&map);

        assert!(res.status == Status::Met);

        // Met & NotMet == NotMet
        root = and(vec![int_equals("foo", 2), string_equals("bar", "bar")]);
        res = root.check_map(&map);

        assert!(res.status == Status::NotMet);

        // Met & Unknown == Unknown
        root = and(vec![int_equals("quux", 2), string_equals("bar", "bar")]);
        res = root.check_map(&map);

        assert!(res.status == Status::Unknown);

        // NotMet & Unknown == NotMet
        root = and(vec![int_equals("quux", 2), string_equals("bar", "baz")]);
        res = root.check_map(&map);

        assert!(res.status == Status::NotMet);

        // Unknown & Unknown == Unknown
        root = and(vec![int_equals("quux", 2), string_equals("fizz", "bar")]);
        res = root.check_map(&map);

        assert!(res.status == Status::Unknown);
    }

    #[test]
    fn or_rules() {
        let map = get_test_data();
        // Met | Met == Met
        let mut root = or(vec![int_equals("foo", 1), string_equals("bar", "bar")]);
        let mut res = root.check_map(&map);

        assert!(res.status == Status::Met);

        // Met | NotMet == Met
        root = or(vec![int_equals("foo", 2), string_equals("bar", "bar")]);
        res = root.check_map(&map);

        assert!(res.status == Status::Met);

        // Met | Unknown == Met
        root = or(vec![int_equals("quux", 2), string_equals("bar", "bar")]);
        res = root.check_map(&map);

        assert!(res.status == Status::Met);

        // NotMet | Unknown == Unknown
        root = or(vec![int_equals("quux", 2), string_equals("bar", "baz")]);
        res = root.check_map(&map);

        assert!(res.status == Status::Unknown);

        // Unknown | Unknown == Unknown
        root = or(vec![int_equals("quux", 2), string_equals("fizz", "bar")]);
        res = root.check_map(&map);

        assert!(res.status == Status::Unknown);
    }

    #[test]
    fn n_of_rules() {
        let map = get_test_data();
        // 2 Met, 1 NotMet == Met
        let mut root = at_least(
            2,
            vec![
                int_equals("foo", 1),
                string_equals("bar", "bar"),
                bool_equals("baz", false),
            ],
        );
        let mut res = root.check_map(&map);

        assert!(res.status == Status::Met);

        // 1 Met, 1 NotMet, 1 Unknown == NotMet
        root = at_least(
            2,
            vec![
                int_equals("foo", 1),
                string_equals("quux", "bar"),
                bool_equals("baz", false),
            ],
        );
        res = root.check_map(&map);

        assert!(res.status == Status::NotMet);

        // 2 NotMet, 1 Unknown == Unknown
        root = at_least(
            2,
            vec![
                int_equals("foo", 2),
                string_equals("quux", "baz"),
                bool_equals("baz", false),
            ],
        );
        res = root.check_map(&map);

        assert!(res.status == Status::NotMet);
    }

    #[test]
    fn string_equals_rule() {
        let map = get_test_data();
        let mut rule = string_equals("bar", "bar");
        let mut res = rule.check_map(&map);
        assert!(res.status == Status::Met);

        rule = string_equals("bar", "baz");
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);
    }

    #[test]
    fn int_equals_rule() {
        let map = get_test_data();
        let mut rule = int_equals("foo", 1);
        let mut res = rule.check_map(&map);
        assert!(res.status == Status::Met);

        rule = int_equals("foo", 2);
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);

        // Values not convertible to int should be NotMet
        rule = int_equals("bar", 2);
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);
    }

    #[test]
    fn int_range_rule() {
        let map = get_test_data();
        let mut rule = int_in_range("foo", 1, 3);
        let mut res = rule.check_map(&map);
        assert!(res.status == Status::Met);

        rule = int_in_range("foo", 2, 3);
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);

        // Values not convertible to int should be NotMet
        rule = int_in_range("bar", 1, 3);
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);
    }

    #[test]
    fn boolean_rule() {
        let mut map = get_test_data();
        let mut rule = bool_equals("baz", true);
        let mut res = rule.check_map(&map);
        assert!(res.status == Status::Met);

        rule = bool_equals("baz", false);
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);

        rule = bool_equals("bar", true);
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);

        rule = bool_equals("bar", false);
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);

        map.insert("quux".into(), "tRuE".into());
        rule = bool_equals("quux", true);
        res = rule.check_map(&map);
        assert!(res.status == Status::NotMet);
    }
}
