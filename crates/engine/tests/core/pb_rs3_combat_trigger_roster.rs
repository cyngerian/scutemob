//! PB-RS3 roster sweep: which card defs carry `TriggerCondition::AtBeginningOfCombat`
//! (the trigger condition `begin_combat`'s card-def sweep now dispatches -- OOS-OS9-1).
//!
//! **Enumerated from `all_cards()`, not grep (SR-36).** The plan's grep baseline (code
//! sites only, comments excluded) found exactly 6 files. This test is the measured
//! deliverable, per the same pattern as `pb_rs1_roster_sweep.rs`.
//!
//! **Field-scoped string match, not a bare `contains_key` walk.**
//! `pb_rs1_roster_sweep.rs`'s `contains_key` helper matches `Effect` variants that
//! carry data (e.g. `Effect::Scry { amount }` serializes externally-tagged as
//! `{"Scry": {...}}`, an object key). `TriggerCondition::AtBeginningOfCombat` is a UNIT
//! variant (no fields), which serde's default external tagging serializes as a bare
//! JSON STRING `"AtBeginningOfCombat"`, not an object key -- a `contains_key`-only walk
//! finds zero matches here even though every roster member's `abilities` field carries
//! this exact value.
//!
//! Worse: a bare "does this string appear anywhere" walk over-counts. `TriggerEvent`
//! (used by `TriggeredAbilityDef::trigger_on`, the EMBLEM path) has its own unit
//! variant with the IDENTICAL name `AtBeginningOfCombat` -- indistinguishable from
//! `TriggerCondition::AtBeginningOfCombat` by string content alone. A naive walk finds
//! `Basri Ket` (whose `-6` ability's emblem carries `trigger_on:
//! TriggerEvent::AtBeginningOfCombat`, `basri_ket.rs:78`) as a false positive -- verified
//! empirically while writing this test. The fix is to scope the match to the JSON
//! OBJECT KEY that specifically carries a `TriggerCondition` in this DSL:
//! `AbilityDefinition::Triggered`'s `trigger_condition` field
//! (`card_definition.rs:338-339`). `TriggeredAbilityDef`'s emblem-event field is named
//! `trigger_on`, a different key -- so scoping to `trigger_condition` is exact, still
//! walks recursively (back faces, modes, nested ability lists), and does not collide
//! with the emblem path.

use mtg_engine::all_cards;
use mtg_engine::cards::card_definition::Completeness;

/// True if the JSON object key `field` appears anywhere in the value tree with a value
/// equal to the target unit-variant string `variant` (or, for a future data-carrying
/// variant, an object keyed by `variant`).
fn contains_field_with_variant(v: &serde_json::Value, field: &str, variant: &str) -> bool {
    match v {
        serde_json::Value::Object(map) => map.iter().any(|(k, child)| {
            if k == field {
                match child {
                    serde_json::Value::String(s) => s == variant,
                    serde_json::Value::Object(inner) => inner.contains_key(variant),
                    _ => false,
                }
            } else {
                contains_field_with_variant(child, field, variant)
            }
        }),
        serde_json::Value::Array(items) => items
            .iter()
            .any(|i| contains_field_with_variant(i, field, variant)),
        _ => false,
    }
}

/// Emits the full sorted roster plus a per-card completeness readout, asserting a
/// non-vacuity floor (the `effect_choose_gate.rs::stub_gates_are_not_vacuous` hazard --
/// a serde rename or a walk that stops finding nesting sites must not silently report
/// nothing) AND pinning the post-PB-RS3 completeness of every named member, so a
/// regression on any of the six (e.g. a later change accidentally reverting
/// `helm_of_the_host` back to a bare `#[default]` or flipping `loyal_apprentice` back to
/// `partial`) fails loudly here instead of only showing up as a coverage-percentage dip.
#[test]
fn pb_rs3_combat_trigger_roster_reports_affected_cards() {
    let defs = all_cards();

    let mut roster: Vec<(String, Completeness)> = Vec::new();
    for def in &defs {
        let json = serde_json::to_value(def).expect("CardDefinition serializes");
        if contains_field_with_variant(&json, "trigger_condition", "AtBeginningOfCombat") {
            roster.push((def.name.clone(), def.completeness.clone()));
        }
    }
    roster.sort_by(|a, b| a.0.cmp(&b.0));

    eprintln!("PB-RS3 AtBeginningOfCombat roster sweep (from all_cards(), not grep):");
    eprintln!("  TOTAL distinct cards: {}", roster.len());
    for (name, completeness) in &roster {
        eprintln!("    {name}: {completeness:?}");
    }

    // Non-vacuity floor: the plan's grep baseline (code sites, comments excluded) found
    // exactly 6 distinct card-def files. `basri_ket.rs` uses the EMBLEM path
    // (`TriggerEvent::AtBeginningOfCombat`, a different enum) and is deliberately
    // EXCLUDED from this roster -- it is the fixture for the emblem-coexistence test,
    // not a card-def-sweep member.
    assert!(
        roster.len() >= 6,
        "roster sweep reports only {} affected cards -- expected at least 6 (the plan's \
         grep baseline was exactly 6 distinct files: helm_of_the_host, loyal_apprentice, \
         siege_gang_lieutenant, goblin_rabblemaster, legion_warboss, mirage_phalanx). This \
         floor exists so a serde rename or a walk that stops finding nesting sites cannot \
         silently pass while reporting near-zero. Full list: {:?}",
        roster.len(),
        roster
    );

    // Pin exact completeness for each of the six known roster members (plan §7 table,
    // corrected against what step 4 of primitive-wip.md actually shipped -- see the
    // PB-RS3 close-out for the discrepancy between the plan's PREDICTED table and what
    // was ACTUALLY authored: goblin_rabblemaster was flipped to Complete too, a third
    // legitimate flip beyond the plan's predicted two).
    let expect_complete = [
        "Helm of the Host",
        "Loyal Apprentice",
        "Siege-Gang Lieutenant",
        "Goblin Rabblemaster",
    ];
    for name in expect_complete {
        let entry = roster
            .iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("known roster member '{name}' is missing -- the walk is broken, not just under-counting. Full list: {roster:?}"));
        assert!(
            matches!(entry.1, Completeness::Complete),
            "'{name}' should be Completeness::Complete after PB-RS3 -- found {:?}",
            entry.1
        );
    }

    let entry = roster
        .iter()
        .find(|(n, _)| n == "Legion Warboss")
        .expect("Legion Warboss should be in the roster");
    assert!(
        matches!(entry.1, Completeness::Partial(_)),
        "'Legion Warboss' should stay Completeness::Partial (Mentor keyword + \
         'attacks this combat if able' both still unimplemented) -- found {:?}",
        entry.1
    );

    let entry = roster
        .iter()
        .find(|(n, _)| n == "Mirage Phalanx")
        .expect("Mirage Phalanx should be in the roster");
    assert!(
        matches!(entry.1, Completeness::KnownWrong(_)),
        "'Mirage Phalanx' should stay Completeness::KnownWrong (now wrong in BOTH \
         directions per the PB-RS3 review -- unpaired over-produces, paired \
         under-produces) -- found {:?}",
        entry.1
    );

    // Sanity: Basri Ket (the emblem-path fixture) must NOT be in this roster -- it uses
    // TriggerEvent::AtBeginningOfCombat (a different enum), not TriggerCondition.
    assert!(
        !roster.iter().any(|(n, _)| n == "Basri Ket"),
        "'Basri Ket' uses the EMBLEM TriggerEvent path, not a card-def TriggerCondition \
         -- it should not appear in this roster. If it does, the walk is matching the \
         wrong enum."
    );
}
