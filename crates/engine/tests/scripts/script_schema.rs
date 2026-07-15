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
            retirement_reason: None,
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

// ── SR-22: unknown keys are rejected on the structural spine ────────────────────

/// A JSON blob that is a valid `GameScript` except for the injected `extra` key.
/// `injection` is spliced in verbatim so each test names exactly the stray key it
/// is proving gets rejected.
fn game_script_json_with(injection: &str) -> String {
    format!(
        r#"{{
          "schema_version": "1.0.0",
          {injection}
          "metadata": {{
            "id": "script_test_deny",
            "name": "deny test",
            "description": "d",
            "cr_sections_tested": [],
            "corner_case_ref": null,
            "tags": [],
            "confidence": "high",
            "review_status": "approved",
            "reviewed_by": null,
            "review_date": null,
            "generation_notes": null
          }},
          "initial_state": {{
            "format": "commander",
            "turn_number": 1,
            "active_player": "p1",
            "phase": "precombat_main",
            "step": null,
            "priority": "p1",
            "players": {{}},
            "zones": {{}}
          }},
          "script": []
        }}"#
    )
}

#[test]
/// The exact bug SR-22 found: a stray *top-level* `review_status` (the shape
/// `stack/135` shipped) is now a hard error, not a silently-dropped key. Without
/// `deny_unknown_fields` on `GameScript` this parses clean and the duplicate value
/// is discarded.
fn stray_top_level_key_is_rejected() {
    // Sanity: the same document *without* the stray key parses.
    let ok = game_script_json_with("");
    serde_json::from_str::<GameScript>(&ok).expect("control document must parse");

    let bad = game_script_json_with(r#""review_status": "approved","#);
    let err = serde_json::from_str::<GameScript>(&bad)
        .expect_err("a stray top-level `review_status` must be rejected (SR-22)");
    assert!(
        err.to_string().contains("review_status"),
        "error should name the offending key, got: {err}"
    );
}

#[test]
/// Strictness reaches into the spine, not just the top level: a key that no
/// `InitialState` field claims is rejected. `mana_paid` is the real stray this
/// test mirrors — `stack/006` had it misplaced inside a player's init block.
fn stray_key_inside_initial_state_is_rejected() {
    let bad = r#"{
      "schema_version": "1.0.0",
      "metadata": {
        "id": "s", "name": "n", "description": "d", "cr_sections_tested": [],
        "corner_case_ref": null, "tags": [], "confidence": "high",
        "review_status": "approved", "reviewed_by": null, "review_date": null,
        "generation_notes": null
      },
      "initial_state": {
        "format": "commander", "turn_number": 1, "active_player": "p1",
        "phase": "precombat_main", "step": null, "priority": "p1",
        "players": {}, "zones": {},
        "bogus_field": 7
      },
      "script": []
    }"#;
    let err = serde_json::from_str::<GameScript>(bad)
        .expect_err("a stray key inside `initial_state` must be rejected (SR-22)");
    assert!(
        err.to_string().contains("bogus_field"),
        "error should name the offending key, got: {err}"
    );
}
