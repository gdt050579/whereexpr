use crate::Condition;
use crate::ConditionList;
use crate::Operation;
use crate::Predicate;

fn sample_condition() -> Condition {
    Condition::new(0, Predicate::with_value(Operation::Is, 0u8).unwrap())
}

#[test]
fn new_is_empty() {
    let list = ConditionList::new();
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn with_capacity_starts_empty() {
    let list = ConditionList::with_capacity(64);
    assert!(list.is_empty());
    assert_eq!(list.len(), 0);
}

#[test]
fn add_inserts_and_returns_true() {
    let mut list = ConditionList::new();
    assert!(list.add("alpha", sample_condition()));
    assert_eq!(list.len(), 1);
    assert!(!list.is_empty());
    assert_eq!(list.from_name("alpha"), Some(0));
    assert!(list.get(0).is_some());
}

#[test]
fn add_duplicate_name_returns_false() {
    let mut list = ConditionList::new();
    assert!(list.add("x", sample_condition()));
    assert!(!list.add("x", sample_condition()));
    assert_eq!(list.len(), 1);
    assert_eq!(list.from_name("x"), Some(0));
}

#[test]
fn names_are_case_insensitive_for_identity() {
    let mut list = ConditionList::new();
    assert!(list.add("Status", sample_condition()));
    assert!(!list.add("status", sample_condition()));
    assert_eq!(list.len(), 1);
    assert_eq!(list.from_name("STATUS"), Some(0));
}

#[test]
fn from_name_unknown_returns_none() {
    let list = ConditionList::new();
    assert_eq!(list.from_name("missing"), None);
}

#[test]
fn indices_match_insertion_order() {
    let mut list = ConditionList::new();
    assert!(list.add("first", sample_condition()));
    assert!(list.add("second", sample_condition()));
    assert_eq!(list.from_name("first"), Some(0));
    assert_eq!(list.from_name("second"), Some(1));
    assert!(list.get(0).is_some());
    assert!(list.get(1).is_some());
}

#[test]
fn get_out_of_range_returns_none() {
    let mut list = ConditionList::new();
    assert!(list.add("only", sample_condition()));
    assert!(list.get(0).is_some());
    assert!(list.get(1).is_none());
}

#[test]
fn many_adds_linear_then_sorted_index() {
    let mut list = ConditionList::new();
    for i in 0..50 {
        let name = format!("n{i}");
        assert!(list.add(&name, sample_condition()), "add {name}");
    }
    assert_eq!(list.len(), 50);
    for i in 0..50 {
        let name = format!("n{i}");
        assert_eq!(list.from_name(&name), Some(i as u16), "lookup {name}");
        assert!(list.get(i as u16).is_some());
    }
}
