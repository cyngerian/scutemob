/// Tests for the GameScript schema type.
///
/// CR reference: n/a — this is test infrastructure, not a rules implementation.
/// Validates that the schema serializes and deserializes correctly (Hook 1 acceptance
/// criterion from `docs/mtg-engine-game-scripts.md`).
use mtg_engine::testing::script_schema::*;
use std::collections::HashMap;

/// Verifies that a minimal `GameScript` round-trips through JSON without loss.
#[test]
fn test_script_schema_round_trip_minimal() {
    let script = GameScript {
        schema_version: "1.0.0".into(),
        metadata: ScriptMetadata {
            id: "script_test_001".into(),
            name: "Round-trip test".into(),
            description: "Schema validation only.".into(),
            cr_sections_tested: vec!["601.2".into()],
            corner_case_ref: None,
            tags: vec!["test".into()],
            confidence: Confidence::High,
            review_status: ReviewStatus::PendingReview,
            reviewed_by: None,
            review_date: None,
            generation_notes: None,
            disputes: vec![],
        },
        initial_state: InitialState {
            format: "commander".into(),
            turn_number: 1,
            active_player: "alice".into(),
            phase: "precombat_main".into(),
            step: None,
            priority: "alice".into(),
            players: HashMap::new(),
            zones: ZonesInitState {
                battlefield: HashMap::new(),
                hand: HashMap::new(),
                graveyard: HashMap::new(),
                exile: vec![],
                command_zone: HashMap::new(),
                library: HashMap::new(),
                stack: vec![],
            },
            continuous_effects: vec![],
        },
        script: vec![],
    };

    let json = serde_json::to_string(&script).expect("serialization failed");
    let decoded: GameScript = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(script, decoded);
}

/// Verifies that all `ScriptAction` variants serialize with the correct `"type"` tag.
#[test]
fn test_script_action_type_tags() {
    let cases: &[(&str, ScriptAction)] = &[
        (
            "priority_pass",
            ScriptAction::PriorityPass {
                player: "alice".into(),
                note: None,
            },
        ),
        (
            "sba_check",
            ScriptAction::SbaCheck {
                results: vec![],
                triggered_abilities: vec![],
                note: None,
            },
        ),
        (
            "assert_state",
            ScriptAction::AssertState {
                description: "test".into(),
                assertions: HashMap::new(),
                note: None,
            },
        ),
        (
            "phase_transition",
            ScriptAction::PhaseTransition {
                from_step: "precombat_main".into(),
                to_step: "beginning_of_combat".into(),
                cr_ref: Some("500.1".into()),
                note: None,
            },
        ),
    ];

    for (expected_tag, action) in cases {
        let json = serde_json::to_string(action).expect("serialization failed");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("deserialization to Value failed");
        assert_eq!(
            value["type"].as_str().unwrap(),
            *expected_tag,
            "wrong type tag for action"
        );
        // Round-trip
        let decoded: ScriptAction =
            serde_json::from_str(&json).expect("round-trip deserialization failed");
        assert_eq!(action, &decoded);
    }
}

/// Verifies that `ReviewStatus` and `Confidence` serialize in snake_case.
#[test]
fn test_enums_serialize_snake_case() {
    assert_eq!(
        serde_json::to_string(&ReviewStatus::PendingReview).unwrap(),
        r#""pending_review""#
    );
    assert_eq!(
        serde_json::to_string(&ReviewStatus::Approved).unwrap(),
        r#""approved""#
    );
    assert_eq!(
        serde_json::to_string(&Confidence::High).unwrap(),
        r#""high""#
    );
    assert_eq!(
        serde_json::to_string(&Confidence::Medium).unwrap(),
        r#""medium""#
    );
}
