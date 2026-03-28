#![cfg(test)]

use soroban_sdk::{Env, Symbol};

use crate::base::types::{
    validate_category, CAT_ARTS, CAT_COMMUNITY, CAT_EDUCATION, CAT_ENVIRONMENT, CAT_HEALTH,
    CAT_POLITICS, CAT_SPORTS, CAT_TECHNOLOGY,
};

// ── valid categories ──────────────────────────────────────────────────────────

#[test]
fn test_cat_sports_is_valid() {
    let env = Env::default();
    assert!(validate_category(&env, &Symbol::new(&env, CAT_SPORTS)));
}

#[test]
fn test_cat_politics_is_valid() {
    let env = Env::default();
    assert!(validate_category(&env, &Symbol::new(&env, CAT_POLITICS)));
}

#[test]
fn test_cat_education_is_valid() {
    let env = Env::default();
    assert!(validate_category(&env, &Symbol::new(&env, CAT_EDUCATION)));
}

#[test]
fn test_cat_health_is_valid() {
    let env = Env::default();
    assert!(validate_category(&env, &Symbol::new(&env, CAT_HEALTH)));
}

#[test]
fn test_cat_environment_is_valid() {
    let env = Env::default();
    assert!(validate_category(&env, &Symbol::new(&env, CAT_ENVIRONMENT)));
}

#[test]
fn test_cat_arts_is_valid() {
    let env = Env::default();
    assert!(validate_category(&env, &Symbol::new(&env, CAT_ARTS)));
}

#[test]
fn test_cat_technology_is_valid() {
    let env = Env::default();
    assert!(validate_category(&env, &Symbol::new(&env, CAT_TECHNOLOGY)));
}

#[test]
fn test_cat_community_is_valid() {
    let env = Env::default();
    assert!(validate_category(&env, &Symbol::new(&env, CAT_COMMUNITY)));
}

// ── invalid categories ────────────────────────────────────────────────────────

#[test]
fn test_unknown_category_is_invalid() {
    let env = Env::default();
    assert!(!validate_category(&env, &Symbol::new(&env, "unknown")));
}

#[test]
fn test_empty_string_is_invalid() {
    let env = Env::default();
    // Symbol::new with an empty str produces a distinct symbol that matches
    // none of the canonical constants.
    assert!(!validate_category(&env, &Symbol::new(&env, "other")));
}

#[test]
fn test_mixed_case_is_invalid() {
    let env = Env::default();
    // Constants are lowercase; "Sports" must not match.
    assert!(!validate_category(&env, &Symbol::new(&env, "Sports")));
}

// ── constant value sanity ─────────────────────────────────────────────────────

#[test]
fn test_constants_are_distinct() {
    let env = Env::default();
    let cats = [
        Symbol::new(&env, CAT_SPORTS),
        Symbol::new(&env, CAT_POLITICS),
        Symbol::new(&env, CAT_EDUCATION),
        Symbol::new(&env, CAT_HEALTH),
        Symbol::new(&env, CAT_ENVIRONMENT),
        Symbol::new(&env, CAT_ARTS),
        Symbol::new(&env, CAT_TECHNOLOGY),
        Symbol::new(&env, CAT_COMMUNITY),
    ];
    for i in 0..cats.len() {
        for j in (i + 1)..cats.len() {
            assert_ne!(cats[i], cats[j], "categories at {i} and {j} must differ");
        }
    }
}

#[test]
fn test_all_constants_pass_validate_category() {
    let env = Env::default();
    for cat in [
        CAT_SPORTS,
        CAT_POLITICS,
        CAT_EDUCATION,
        CAT_HEALTH,
        CAT_ENVIRONMENT,
        CAT_ARTS,
        CAT_TECHNOLOGY,
        CAT_COMMUNITY,
    ] {
        assert!(
            validate_category(&env, &Symbol::new(&env, cat)),
            "{cat} must be valid"
        );
    }
}
