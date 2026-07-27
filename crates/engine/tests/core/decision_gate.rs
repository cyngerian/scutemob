//! PB-DP10: the invariant-level gate for the whole PB-DP suite.
//!
//! `effect_choose_gate.rs` (SR-33/34/37/38 + PB-EF12) bars exactly three DSL variants
//! (`Effect::Choose`, `Effect::MayPayOrElse`, `Effect::AddManaChoice` + the any-color
//! family) from `Complete`. `docs/audits/decision-point-audit.md` §3.1 counts 21 rows
//! across 277 of 1,139 effectively-`Complete` defs (24.3%) where the engine makes a
//! player's choice for them; 17 of those rows the gate never named, so the figure grows
//! silently with every card authored.
//!
//! This file does not close that gap at the engine level -- that is what PB-DP1..DP9
//! were for, and what the still-open rows (DP-13/14/16/17/18/19/20/25/26/31) remain for.
//! It converts **silent** growth into **recorded, reviewed** growth: a `Complete` def
//! newly carrying a still-auto-chosen decision fails `T4` until its author either demotes
//! it or adds a `BASELINE` entry with a written reason. **This gate cannot stop the
//! growth; it makes it recorded** (see `T4`'s failure message, `t4_failure_message_names_
//! the_bound`, and the audit's own PB-DP10 row -- do not read this file as a closure of
//! DP-INV).
//!
//! No engine or wire change: PROTOCOL 31 / HASH 68 are unmoved by this batch (`crates/
//! engine/tests/core/protocol_schema.rs` / `hash_schema.rs`'s `SCAN_ROOTS` never reach
//! `crates/engine/tests/`).

use crate::decision_site_walk::{
    auto_chosen_row_hits, is_effectively_complete, row_hits, DecisionClass, PROSE_FIELDS, ROWS,
};
use mtg_engine::cards::card_definition::{
    LibraryPosition, ModeSelection, WheelDisposal, WheelDraw,
};
use mtg_engine::{
    all_cards, AbilityDefinition, CardEffectTarget as EffectTarget, Cost, Effect, EffectAmount,
    ManaColor, PlayerTarget, SubType, TargetFilter, TargetRequirement, TriggerCondition,
    ZoneTarget,
};
use std::collections::{BTreeSet, HashMap};

fn defs_map() -> HashMap<String, mtg_engine::CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

// ── T1: every row predicate is non-vacuous ────────────────────────────────────

/// A minimal, real DSL value that carries the row's own site (an `Effect` or
/// `AbilityDefinition` node, serialized directly -- `Row::predicate` takes any
/// `serde_json::Value`, not specifically a whole `CardDefinition`, so this is a faithful
/// exercise of the same walk a `CardDefinition`'s serialization would produce).
fn positive_value_for_row(id: &str) -> serde_json::Value {
    let v = match id {
        "triggered_targets" => serde_json::to_value(AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::Nothing,
            intervening_if: None,
            targets: vec![TargetRequirement::TargetPlayer],
            modes: None,
            trigger_zone: None,
            once_per_turn: false,
        }),
        "search_library" => serde_json::to_value(Effect::SearchLibrary {
            player: PlayerTarget::Controller,
            filter: TargetFilter::default(),
            reveal: false,
            destination: ZoneTarget::Hand {
                owner: PlayerTarget::Controller,
            },
            shuffle_before_placing: false,
            also_search_graveyard: false,
        }),
        "proliferate" => serde_json::to_value(Effect::Proliferate),
        "discard_cards" => serde_json::to_value(Effect::DiscardCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        "wheel_hand" => serde_json::to_value(Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::Discard,
            draw: WheelDraw::Fixed(1),
        }),
        "scry" => serde_json::to_value(Effect::Scry {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        "sacrifice_permanents" => serde_json::to_value(Effect::SacrificePermanents {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: None,
        }),
        "may_pay_then_effect" => serde_json::to_value(Effect::MayPayThenEffect {
            cost: Cost::Tap,
            payer: PlayerTarget::Controller,
            then: Box::new(Effect::Nothing),
        }),
        "choose_color_or_type" => serde_json::to_value(Effect::ChooseCreatureType {
            default: SubType("Human".to_string()),
        }),
        "look_at_top_or_route" => serde_json::to_value(Effect::LookAtTopThenPlace {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: TargetFilter::default(),
            place_cost: None,
            destination: ZoneTarget::Hand {
                owner: PlayerTarget::Controller,
            },
            rest_to: ZoneTarget::Library {
                owner: PlayerTarget::Controller,
                position: LibraryPosition::Bottom,
            },
            optional: false,
        }),
        "surveil" => serde_json::to_value(Effect::Surveil {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        "counter_unless_pays" => serde_json::to_value(Effect::CounterUnlessPays {
            target: EffectTarget::DeclaredTarget { index: 0 },
            cost: Cost::Tap,
        }),
        "modal_trigger" => serde_json::to_value(AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::Nothing,
            intervening_if: None,
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                modes: vec![Effect::Nothing],
                allow_duplicate_modes: false,
                mode_costs: None,
                mode_targets: None,
            }),
            trigger_zone: None,
            once_per_turn: false,
        }),
        "change_targets" => serde_json::to_value(Effect::ChangeTargets {
            target: EffectTarget::Source,
            must_change: false,
        }),
        "put_on_library" => serde_json::to_value(Effect::PutOnLibrary {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            from: ZoneTarget::Hand {
                owner: PlayerTarget::Controller,
            },
        }),
        "bolster_amass" => serde_json::to_value(Effect::Bolster {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        "connive" => serde_json::to_value(Effect::Connive {
            target: EffectTarget::Source,
            count: EffectAmount::Fixed(1),
        }),
        "discover" => serde_json::to_value(Effect::Discover {
            player: PlayerTarget::Controller,
            n: 1,
        }),
        "may_pay_or_else" => serde_json::to_value(Effect::MayPayOrElse {
            cost: Cost::Tap,
            payer: PlayerTarget::Controller,
            or_else: Box::new(Effect::Nothing),
        }),
        "add_mana_filter_choice" => serde_json::to_value(Effect::AddManaFilterChoice {
            player: PlayerTarget::Controller,
            color_a: ManaColor::White,
            color_b: ManaColor::Blue,
        }),
        "choose_stub" => serde_json::to_value(Effect::Choose {
            prompt: "probe".to_string(),
            choices: vec![Effect::Nothing],
        }),
        "the_ring_tempts_you" => serde_json::to_value(Effect::TheRingTemptsYou),
        other => panic!("no positive probe registered for row id {other:?}"),
    };
    v.expect("DSL node serializes")
}

/// A universal negative: `Effect::Nothing` is a unit variant matching none of the 22 row
/// keys/strings and is not an object, so the two compound (`Triggered`-qualified) row
/// predicates correctly find nothing to inspect either.
fn negative_value() -> serde_json::Value {
    serde_json::to_value(Effect::Nothing).unwrap()
}

#[test]
/// Each of the 22 rows: a positive probe carrying the row's own site is detected by that
/// row's predicate; the shared negative probe is not. At least one row (`proliferate`) is
/// also probed NESTED two levels deep inside `Sequence(Sequence(..))` -- the whole reason
/// this is a serde tree walk and not a top-level match (mirrors
/// `effect_choose_gate.rs::stub_gates_are_not_vacuous`).
fn every_decision_row_predicate_is_non_vacuous() {
    let neg = negative_value();
    for row in ROWS {
        let pos = positive_value_for_row(row.id);
        assert!(
            (row.predicate)(&pos),
            "row {:?}'s own positive probe was not detected by its predicate: {pos:?}",
            row.id
        );
        assert!(
            !(row.predicate)(&neg),
            "row {:?}'s predicate flagged a bare Effect::Nothing probe -- it is not vacuously true",
            row.id
        );
    }

    // The nesting proof: Sequence(Sequence(Proliferate)).
    let nested = serde_json::to_value(Effect::Sequence(vec![Effect::Sequence(vec![
        Effect::Proliferate,
    ])]))
    .unwrap();
    let proliferate_row = ROWS
        .iter()
        .find(|r| r.id == "proliferate")
        .expect("proliferate row exists");
    assert!(
        (proliferate_row.predicate)(&nested),
        "the proliferate row's predicate must detect Effect::Proliferate nested two levels \
         deep inside Sequence(Sequence(..))"
    );
}

// ── T2: the unit-variant hole, pinned in both directions ──────────────────────

/// The legacy walk every prior gate used (`effect_choose_gate.rs::contains_key`,
/// `pb_rs1_roster_sweep.rs::contains_key`,
/// `primitives/pb_dp9_effect_choice.rs::roster::json_contains_variant`): object keys only.
fn legacy_object_key_only_contains(v: &serde_json::Value, key: &str) -> bool {
    match v {
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(k, child)| k == key || legacy_object_key_only_contains(child, key)),
        serde_json::Value::Array(items) => items
            .iter()
            .any(|i| legacy_object_key_only_contains(i, key)),
        _ => false,
    }
}

#[test]
/// **The batch's central fail-before.** A unit `Effect` variant (`Proliferate`,
/// `TheRingTemptsYou`) serializes to a bare JSON STRING, not an object key -- pinned
/// directly against `serde_json::to_value` first, then shown to defeat every walk that
/// predates this one. `crate::decision_site_walk::json_contains_variant` (which IS what
/// every `Row::predicate` above is built on) must still find it. Written so a future
/// maintainer who "simplifies" the walk back to object-keys-only gets a red test naming
/// the bug, not a silently-vacuous green gate.
fn unit_variant_rows_need_string_matching() {
    let prolif = serde_json::to_value(Effect::Proliferate).unwrap();
    assert_eq!(
        prolif,
        serde_json::Value::String("Proliferate".to_string()),
        "a unit Effect variant must serialize to a bare JSON string, not an object key -- \
         this is the premise the whole walk design rests on (plan §2.1/§11 P4)"
    );
    let ring = serde_json::to_value(Effect::TheRingTemptsYou).unwrap();
    assert_eq!(
        ring,
        serde_json::Value::String("TheRingTemptsYou".to_string())
    );

    assert!(
        !legacy_object_key_only_contains(&prolif, "Proliferate"),
        "the legacy object-key-only walk must NOT see a bare unit-variant string -- if this \
         assertion ever fails, `Effect::Proliferate`'s serde shape changed and every row \
         predicate built on json_contains_variant needs re-auditing"
    );
    assert!(
        crate::decision_site_walk::json_contains_variant(&prolif, "Proliferate"),
        "the CANONICAL walk (json_contains_variant) must see the bare unit-variant string \
         the legacy walk is blind to -- this is the fix, not merely the bug report"
    );

    // Nested inside a real card-shaped tree, not just the bare Effect value.
    let nested = serde_json::to_value(Effect::Sequence(vec![Effect::Proliferate])).unwrap();
    assert!(
        !legacy_object_key_only_contains(&nested, "Proliferate"),
        "the legacy walk must also miss it nested inside Sequence(..)"
    );
    assert!(crate::decision_site_walk::json_contains_variant(
        &nested,
        "Proliferate"
    ));
}

// ── T3: PROSE_FIELDS denylist, both directions ────────────────────────────────

#[test]
/// A bare `Value::String("Proliferate")` that is the direct value of a denylisted field
/// key must NOT be treated as a unit-variant hit; the identical string under any OTHER
/// key, or the real `Effect::Proliferate` node nested inside `Sequence(Sequence(..))`,
/// MUST be. Constructed as raw JSON literals rather than a full `CardDefinition` --
/// `json_contains_variant`'s suppression logic is a property of the JSON tree and the
/// parent key, independent of which Rust type produced it, so this exercises the
/// mechanism directly without the ceremony of building a `TriggeredAbilityDef` /
/// `Effect::CreateEmblem` chain just to get a `description` field onto the wire.
fn prose_fields_do_not_trigger_a_unit_variant_row() {
    for field in ["name", "oracle_text", "prompt", "description", "card_id"] {
        assert!(
            PROSE_FIELDS.contains(&field),
            "test setup: {field:?} must be one of the denylisted PROSE_FIELDS"
        );
        let v = serde_json::json!({ field: "Proliferate" });
        assert!(
            !crate::decision_site_walk::json_contains_variant(&v, "Proliferate"),
            "a bare Value::String(\"Proliferate\") under prose field {field:?} must be \
             suppressed, not treated as a unit-variant hit"
        );
    }

    // Negative control: the SAME string under a non-denylisted key is NOT suppressed --
    // proves the denylist is field-specific, not a blanket string-value exemption.
    let non_prose = serde_json::json!({ "some_other_field": "Proliferate" });
    assert!(
        !PROSE_FIELDS.contains(&"some_other_field"),
        "test setup: some_other_field must not be denylisted"
    );
    assert!(crate::decision_site_walk::json_contains_variant(
        &non_prose,
        "Proliferate"
    ));

    // The real positive: Effect::Proliferate nested inside Sequence(Sequence(..)) is
    // still detected even in the presence of a same-string prose field elsewhere in the
    // (synthetic) tree.
    let mixed = serde_json::json!({
        "oracle_text": "Proliferate",
        "effect_tree": {"Sequence": [{"Sequence": ["Proliferate"]}]},
    });
    assert!(
        crate::decision_site_walk::json_contains_variant(&mixed, "Proliferate"),
        "a genuine nested unit-variant hit must still be found even when a prose field \
         elsewhere in the same tree carries the identical string"
    );
}

// ── T4: the gate ───────────────────────────────────────────────────────────────

/// (def name exactly as `all_cards()` reports it, the exact sorted set of AutoChosen row
/// ids it hits, a post-freeze reason). `None` = frozen at the 2026-07-27 PB-DP10 measurement;
/// `Some(text)` = a later, deliberate addition, which must carry its own reason (`T5`).
///
/// Measured by this batch's own `T9`/`decision_site_reconciliation_report`, NOT the audit's
/// estimate (plan §11 P6: every §3.1 count is unverified until this gate prints it).
/// Sorted by name.
const BASELINE: &[(&str, &[&str], Option<&str>)] = &[
    ("Accursed Marauder", &["sacrifice_permanents"], None),
    ("Anowon, the Ruin Sage", &["sacrifice_permanents"], None),
    ("Atomize", &["proliferate"], None),
    ("Atraxa, Praetors' Voice", &["proliferate"], None),
    ("Birthing Ritual", &["look_at_top_or_route"], None),
    ("Blightbelly Rat", &["proliferate"], None),
    ("Bloated Contaminator", &["proliferate"], None),
    ("Bolt Bend", &["change_targets"], None),
    ("Brainstorm", &["put_on_library"], None),
    ("Burglar Rat", &["discard_cards"], None),
    ("Butcher of Malakir", &["sacrifice_permanents"], None),
    ("Cached Defenses", &["bolster_amass"], None),
    ("Caged Sun", &["choose_color_or_type"], None),
    ("Cankerbloom", &["proliferate"], None),
    ("Chaos Warp", &["look_at_top_or_route"], None),
    ("Chart a Course", &["discard_cards"], None),
    ("Coiling Oracle", &["look_at_top_or_route"], None),
    ("Consign // Oblivion", &["discard_cards"], None),
    ("Contagion Clasp", &["proliferate"], None),
    ("Contaminant Grafter", &["proliferate"], None),
    ("Contentious Plan", &["proliferate"], None),
    ("Crippling Fear", &["choose_color_or_type"], None),
    ("Crossway Troublemakers", &["may_pay_then_effect"], None),
    ("Deflecting Swat", &["change_targets"], None),
    ("Demon's Disciple", &["sacrifice_permanents"], None),
    ("Dictate of Erebos", &["sacrifice_permanents"], None),
    ("Disciple of Freyalise", &["may_pay_then_effect"], None),
    ("Dreadhorde Invasion", &["bolster_amass"], None),
    ("Dromoka, the Eternal", &["bolster_amass"], None),
    ("Drown in Ichor", &["proliferate"], None),
    ("Etchings of the Chosen", &["choose_color_or_type"], None),
    ("Evolution Sage", &["proliferate"], None),
    ("Faithless Looting", &["discard_cards"], None),
    ("Felidar Retreat", &["modal_trigger"], None),
    ("Fell Specter", &["discard_cards"], None),
    ("Fleshbag Marauder", &["sacrifice_permanents"], None),
    ("Flusterstorm", &["counter_unless_pays"], None),
    ("Flux Channeler", &["proliferate"], None),
    ("Frantic Search", &["discard_cards"], None),
    ("Geier Reach Sanitarium", &["discard_cards"], None),
    ("Geological Appraiser", &["discover"], None),
    ("Goblin Ringleader", &["look_at_top_or_route"], None),
    ("Grateful Apparition", &["proliferate"], None),
    ("Grave Pact", &["sacrifice_permanents"], None),
    ("Greater Good", &["discard_cards"], None),
    ("Grisly Salvage", &["look_at_top_or_route"], None),
    ("Growing Rites of Itlimoc", &["look_at_top_or_route"], None),
    ("Hazoret's Monument", &["may_pay_then_effect"], None),
    ("Hullbreaker Horror", &["modal_trigger"], None),
    ("Inexorable Tide", &["proliferate"], None),
    (
        "Izzet Charm",
        &["counter_unless_pays", "discard_cards"],
        None,
    ),
    ("Kalastria Highborn", &["may_pay_then_effect"], None),
    ("Karn's Bastion", &["proliferate"], None),
    ("Kindred Dominance", &["choose_color_or_type"], None),
    ("Korvold, Fae-Cursed King", &["sacrifice_permanents"], None),
    ("Leaf-Crowned Visionary", &["may_pay_then_effect"], None),
    ("Make Disappear", &["counter_unless_pays"], None),
    ("Mana Leak", &["counter_unless_pays"], None),
    ("Mana Tithe", &["counter_unless_pays"], None),
    ("Merciless Executioner", &["sacrifice_permanents"], None),
    ("Metastatic Evangel", &["proliferate"], None),
    ("Miara, Thorn of the Glade", &["may_pay_then_effect"], None),
    ("Misdirection", &["change_targets"], None),
    ("Morophon, the Boundless", &["choose_color_or_type"], None),
    ("Nadir Kraken", &["may_pay_then_effect"], None),
    ("Nether Traitor", &["may_pay_then_effect"], None),
    ("Obelisk of Urd", &["choose_color_or_type"], None),
    ("Pact of the Serpent", &["choose_color_or_type"], None),
    ("Patchwork Banner", &["choose_color_or_type"], None),
    ("Pull from Tomorrow", &["discard_cards"], None),
    ("Radstorm", &["proliferate"], None),
    ("Raffine's Informant", &["connive"], None),
    ("Raiders' Wake", &["discard_cards"], None),
    ("Retreat to Kazandu", &["modal_trigger"], None),
    ("Risen Reef", &["look_at_top_or_route"], None),
    ("Roalesk, Apex Hybrid", &["proliferate"], None),
    ("Roiling Regrowth", &["sacrifice_permanents"], None),
    ("Satyr Wayfinder", &["look_at_top_or_route"], None),
    ("Shambling Ghast", &["modal_trigger"], None),
    ("Smuggler's Copter", &["discard_cards"], None),
    ("Spell Pierce", &["counter_unless_pays"], None),
    ("Springbloom Druid", &["may_pay_then_effect"], None),
    ("Staff of Compleation", &["proliferate"], None),
    ("Stubborn Denial", &["counter_unless_pays"], None),
    ("Sword of Feast and Famine", &["discard_cards"], None),
    ("Sword of Truth and Justice", &["proliferate"], None),
    ("Sylvan Messenger", &["look_at_top_or_route"], None),
    (
        "Tainted Observer",
        &["may_pay_then_effect", "proliferate"],
        None,
    ),
    ("Tezzeret's Gambit", &["proliferate"], None),
    ("Thirsting Roots", &["proliferate"], None),
    ("Thrasios, Triton Hero", &["look_at_top_or_route"], None),
    ("Thrummingbird", &["proliferate"], None),
    ("Unnatural Restoration", &["proliferate"], None),
    ("Urza's Incubator", &["choose_color_or_type"], None),
    ("Vanquisher's Banner", &["choose_color_or_type"], None),
    ("Victimize", &["sacrifice_permanents"], None),
    ("Yawgmoth, Thran Physician", &["proliferate"], None),
];

/// Measured on this batch's implementing commit (2026-07-27, PB-DP10, `scutemob-158`) --
/// see the `bare_lookup_ratchet.rs` comment convention. `T9` reprints this number live on
/// every run; if it changes, this constant must be updated in the SAME commit that changes
/// it (either direction), per the ratchet's own rule.
const MAX_AUTO_CHOSEN_COMPLETE_UNION: usize = 97;

const MIN_ROWS: usize = 22;
const MIN_BASELINE: usize = 50;
const MIN_CORPUS: usize = 1000;

fn baseline_map() -> HashMap<&'static str, BTreeSet<&'static str>> {
    BASELINE
        .iter()
        .map(|(name, rows, _)| (*name, rows.iter().copied().collect()))
        .collect()
}

#[test]
/// **The gate.** Every effectively-`Complete` def hitting >= 1 `AutoChosen` row must
/// either be absent (offender) or present in `BASELINE` with an EXACT row-set match
/// (offender on mismatch too -- a superset means the def gained a decision since the
/// freeze, a subset means the entry should be tightened).
fn no_complete_def_introduces_an_unrecorded_auto_chosen_decision() {
    let baseline = baseline_map();
    let defs = all_cards();
    let mut offenders: Vec<String> = Vec::new();

    for def in &defs {
        if !is_effectively_complete(def) {
            continue;
        }
        let hits = auto_chosen_row_hits(def);
        if hits.is_empty() {
            continue;
        }
        match baseline.get(def.name.as_str()) {
            None => {
                let detail: Vec<String> = hits
                    .iter()
                    .map(|id| {
                        let row = ROWS.iter().find(|r| r.id == *id).unwrap();
                        format!("{} hits {id} (CR {}, {})", def.name, row.cr, row.site)
                    })
                    .collect();
                offenders.push(format!(
                    "{} is NOT in BASELINE but hits {:?}. {}",
                    def.name,
                    hits,
                    detail.join("; ")
                ));
            }
            Some(recorded) if recorded != &hits => {
                offenders.push(format!(
                    "{} is in BASELINE with rows {:?} but the engine now sees {:?} \
                     (superset = gained a decision since the freeze; subset = tighten the entry)",
                    def.name, recorded, hits
                ));
            }
            _ => {}
        }
    }

    assert!(
        offenders.is_empty(),
        "These effectively-Complete card defs contain a decision the CR gives to a \
         player and the engine still makes for them (audit DP-INV, \
         docs/audits/decision-point-audit.md §1). The decision is legal -- this is not a \
         rules violation -- but the game history records a choice no player made, which \
         is the same defect Architecture Invariant 9 / SR-2 exist to keep out of a deck.\n\n\
         THIS GATE CANNOT STOP THE GROWTH; IT MAKES IT RECORDED. Two legal exits, and \
         only two:\n\
         1. Mark the def non-Complete with a note naming the auto-chosen decision -- \
            `completeness: Completeness::known_wrong(\"engine chooses which card is \
            discarded (CR 701.9b)\")`.\n\
         2. Add a BASELINE entry in this file with the def's exact row set AND a written \
            reason -- a reviewed acknowledgement that this card ships with the engine \
            choosing for the player until the owning PB lands.\n\n\
         Implementing the choice properly is NOT an exit for this batch: it needs the \
         owning engine PB (docs/audits/decision-point-audit.md §5, rows DP-13..DP-31), \
         not a card-def edit.\n\nOffenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
/// T4 is not vacuously green: a synthetic `Complete` def hitting `proliferate` and
/// absent from `BASELINE` must be flagged by the same offender-detection logic. Proven
/// against an in-memory corpus, not the real one (this must never touch `all_cards()`'s
/// gate).
fn t4_gate_logic_reddens_on_a_new_unbaselined_auto_chosen_complete_def() {
    let mut fake = mtg_engine::CardDefinition {
        name: "PB-DP10 Synthetic Offender".to_string(),
        oracle_text: "Proliferate.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::Proliferate,
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    fake.completeness = mtg_engine::cards::Completeness::Complete;

    let baseline = baseline_map();
    assert!(
        !baseline.contains_key(fake.name.as_str()),
        "test setup: the synthetic name must not collide with a real BASELINE entry"
    );
    let hits = auto_chosen_row_hits(&fake);
    assert!(
        !hits.is_empty() && !baseline.contains_key(fake.name.as_str()),
        "the synthetic def must be a genuine offender under the same predicate T4 uses"
    );
}

// ── T5: every BASELINE entry is live and necessary ────────────────────────────

#[test]
/// Each `BASELINE` entry names a real, still-`Complete` def whose CURRENT row set equals
/// the recorded one exactly. Mirrors
/// `completeness_deviation_scan::every_allowlist_entry_is_live_and_necessary`.
fn every_baseline_entry_is_live_and_necessary() {
    let defs = defs_map();
    for (name, rows, reason) in BASELINE {
        let def = defs
            .get(*name)
            .unwrap_or_else(|| panic!("BASELINE names {name:?}, which is not in all_cards()"));
        assert!(
            is_effectively_complete(def),
            "BASELINE entry {name:?} is no longer Complete -- it passes on the marker now, \
             remove the redundant entry"
        );
        let recorded: BTreeSet<&'static str> = rows.iter().copied().collect();
        let actual = auto_chosen_row_hits(def);
        assert_eq!(
            actual, recorded,
            "BASELINE entry {name:?} recorded rows {recorded:?} but the engine now sees \
             {actual:?} -- a superset means this def gained a decision since the freeze \
             (investigate, don't just widen the entry); a subset means the entry should be \
             tightened to the smaller set"
        );
        if let Some(reason_text) = reason {
            assert!(
                reason_text.len() >= 30,
                "BASELINE entry {name:?}'s post-freeze reason is too short ({} chars); a \
                 reviewed acknowledgement needs a real sentence, not a stub",
                reason_text.len()
            );
        }
    }
}

// ── T6: the union ratchet ──────────────────────────────────────────────────────

#[test]
/// The still-auto union count must equal `MAX_AUTO_CHOSEN_COMPLETE_UNION` exactly -- a new
/// def slotting into a row whose ceiling has slack is exactly the hole T4 alone cannot
/// close (T4 checks per-def presence, not the aggregate). One cause, two red tests: if
/// this fails, check `T4` first -- it will usually name the specific def.
fn auto_chosen_complete_union_is_ratcheted() {
    assert!(
        ROWS.len() >= MIN_ROWS,
        "ROWS shrank to {} (< {MIN_ROWS}) -- rows may be added, never removed",
        ROWS.len()
    );
    assert!(
        BASELINE.len() >= MIN_BASELINE,
        "BASELINE shrank to {} (< {MIN_BASELINE}) -- a collapsed baseline could be hiding a \
         broken predicate rather than genuine authoring progress",
        BASELINE.len()
    );
    let defs = all_cards();
    assert!(
        defs.len() >= MIN_CORPUS,
        "the corpus shrank to {} (< {MIN_CORPUS}) -- denominator guard",
        defs.len()
    );

    let union: BTreeSet<String> = defs
        .iter()
        .filter(|d| is_effectively_complete(d))
        .filter(|d| !auto_chosen_row_hits(d).is_empty())
        .map(|d| d.name.clone())
        .collect();

    if union.len() > MAX_AUTO_CHOSEN_COMPLETE_UNION {
        panic!(
            "the still-auto-chosen Complete union grew to {} from the pinned {}. A new \
             Complete def now carries an engine-made choice T4 has not recorded -- see T4's \
             failure message first (it will usually name the specific def); if T4 is green \
             but this reddens, a def slotted into a row whose per-row slack this union check \
             alone catches. Either demote the new def or add a BASELINE entry, then raise \
             MAX_AUTO_CHOSEN_COMPLETE_UNION to {} in the SAME commit.",
            union.len(),
            MAX_AUTO_CHOSEN_COMPLETE_UNION,
            union.len()
        );
    }
    if union.len() < MAX_AUTO_CHOSEN_COMPLETE_UNION {
        panic!(
            "the still-auto-chosen Complete union shrank to {} from the pinned {} -- good, \
             some defs were demoted or an engine PB served a row. Lower \
             MAX_AUTO_CHOSEN_COMPLETE_UNION to {} (and prune the now-stale BASELINE entries \
             T5 will name) so the ratchet keeps the gain.",
            union.len(),
            MAX_AUTO_CHOSEN_COMPLETE_UNION,
            union.len()
        );
    }
}

// ── T7: the two hard zeros ─────────────────────────────────────────────────────

#[test]
/// `add_mana_filter_choice` (`AddManaFilterChoice`) and `the_ring_tempts_you`
/// (`TheRingTemptsYou`) have zero `Complete` defs today -- held by nothing but
/// hand-authoring discipline until this test existed (plan §4 note 5: the SR-33 gate
/// bars a DIFFERENT key, `AddManaChoice`, and does not reach `AddManaFilterChoice`).
fn hard_zero_rows_have_no_complete_defs() {
    let defs = all_cards();
    for id in ["add_mana_filter_choice", "the_ring_tempts_you"] {
        let row = ROWS.iter().find(|r| r.id == id).unwrap();
        let complete: Vec<String> = defs
            .iter()
            .filter(|d| is_effectively_complete(d))
            .filter(|d| (row.predicate)(&serde_json::to_value(*d).unwrap()))
            .map(|d| d.name.clone())
            .collect();
        assert!(
            complete.is_empty(),
            "row {id:?} was a HAND-MAINTAINED zero until this test -- nothing else holds \
             it (the SR-33 gate bars a different serde key). Now Complete: {complete:?}"
        );
    }
}

// ── T8: served rows still have their hooks (compile-forced) ───────────────────

#[test]
/// If PB-DP8's or PB-DP9's channel were ever reverted, a `Served` row's class would
/// become a lie. Force that possibility to be a COMPILE ERROR, not a stale comment: build
/// the two Commands and call the two `GameState` accessors the served rows depend on.
fn served_rows_still_have_their_hooks() {
    // Compile-forced existence + basic call shape.
    let _cmd1 = mtg_engine::Command::AnswerEffectChoice {
        player: mtg_engine::PlayerId(1),
        choice_id: 0,
        answer: mtg_engine::EffectChoiceAnswer::SearchLibrary { found: None },
    };
    let _cmd2 = mtg_engine::Command::ChooseTriggerTargets {
        player: mtg_engine::PlayerId(1),
        choice_id: 0,
        targets: vec![],
    };

    let registry = mtg_engine::CardRegistry::new(all_cards());
    let state = mtg_engine::GameStateBuilder::new()
        .add_player(mtg_engine::PlayerId(1))
        .add_player(mtg_engine::PlayerId(2))
        .with_registry(registry)
        .active_player(mtg_engine::PlayerId(1))
        .at_step(mtg_engine::Step::PreCombatMain)
        .build()
        .expect("state builds");
    assert!(
        state.pending_effect_choice().is_none(),
        "a fresh state has no outstanding CR 608.2d choice"
    );
    assert!(
        state.pending_trigger_targets().is_none(),
        "a fresh state has no outstanding CR 603.3d trigger-target choice"
    );

    // Each Served row's roster floor is non-zero.
    for (id, min) in [
        ("triggered_targets", 1usize),
        ("search_library", 1),
        ("scry", 1),
        ("surveil", 1),
    ] {
        let row = ROWS.iter().find(|r| r.id == id).unwrap();
        assert!(
            matches!(row.class, DecisionClass::Served { .. }),
            "row {id:?} must be classified Served"
        );
        let count = all_cards()
            .iter()
            .filter(|d| is_effectively_complete(d))
            .filter(|d| (row.predicate)(&serde_json::to_value(*d).unwrap()))
            .count();
        assert!(
            count >= min,
            "served row {id:?} has only {count} Complete defs (< {min}) -- its hook may \
             have gone dark"
        );
    }
}

// ── T9: the reconciliation report ─────────────────────────────────────────────

#[test]
/// Prints per-row Complete / non-Complete counts, the all-rows union (the 277 analogue),
/// the still-auto union (BASELINE's size), and the live effectively-Complete denominator
/// and percentage. **Closes OOS-DP7-7** ("the §3.1 re-derivation is still owed").
/// Assertions are `>=` floors only (the PB-DP9 convention: an `==` pin reddens on
/// unrelated authoring).
fn decision_site_reconciliation_report() {
    let defs = all_cards();
    println!("PB-DP10 decision-site reconciliation (enumerated from all_cards(), not grep):");
    for row in ROWS {
        let mut complete = 0usize;
        let mut other = 0usize;
        for def in &defs {
            if (row.predicate)(&serde_json::to_value(def).unwrap()) {
                if is_effectively_complete(def) {
                    complete += 1;
                } else {
                    other += 1;
                }
            }
        }
        let class_detail = match &row.class {
            DecisionClass::Served { by, residual } => {
                format!("SERVED by {by}; residual seeds: {residual:?}")
            }
            DecisionClass::AutoChosen {
                why_not_flagged_is_wrong,
            } => format!("AUTO-CHOSEN: {why_not_flagged_is_wrong}"),
            DecisionClass::Gated { by } => format!("GATED by {by}"),
            DecisionClass::NoDecision { why } => format!("NO-DECISION: {why}"),
        };
        println!(
            "  {}: {complete} Complete (+{other} non-Complete) -- {class_detail}",
            row.id
        );
    }

    let all_union: BTreeSet<String> = defs
        .iter()
        .filter(|d| is_effectively_complete(d))
        .filter(|d| !row_hits(d).is_empty())
        .map(|d| d.name.clone())
        .collect();
    let auto_union: BTreeSet<String> = defs
        .iter()
        .filter(|d| is_effectively_complete(d))
        .filter(|d| !auto_chosen_row_hits(d).is_empty())
        .map(|d| d.name.clone())
        .collect();
    let complete_total = defs.iter().filter(|d| is_effectively_complete(d)).count();

    println!(
        "  ALL-ROWS UNION (the audit's 277 analogue): {}",
        all_union.len()
    );
    println!("  STILL-AUTO UNION (BASELINE's size): {}", auto_union.len());
    println!(
        "  live denominator: {complete_total} Complete of {} total ({:.1}%)",
        defs.len(),
        100.0 * complete_total as f64 / defs.len().max(1) as f64
    );

    assert!(all_union.len() >= 100, "all-rows union collapsed");
    assert!(auto_union.len() >= 50, "still-auto union collapsed");
    assert!(complete_total >= 500, "Complete denominator collapsed");
}

// ── T10 / T11: cross-target value checks against PB-DP8 / PB-DP9's own rosters ─

#[test]
/// Reproduces PB-DP9's post-fix-cycle published numbers (`search >= 73`, `scry >= 16`,
/// `surveil >= 8`) via THIS canonical walk, as a cross-target value check replacing a
/// textual parity gate (the two `roster` modules live in separate integration-test
/// targets and cannot share code across the SR-9a boundary).
fn canonical_walk_reproduces_pb_dp9_rosters() {
    let defs = all_cards();
    for (id, floor) in [("search_library", 73usize), ("scry", 16), ("surveil", 8)] {
        let row = ROWS.iter().find(|r| r.id == id).unwrap();
        let count = defs
            .iter()
            .filter(|d| is_effectively_complete(d))
            .filter(|d| (row.predicate)(&serde_json::to_value(*d).unwrap()))
            .count();
        assert!(
            count >= floor,
            "row {id:?} has only {count} Complete defs, expected >= {floor} (PB-DP9's own \
             published number)"
        );
    }
}

#[test]
/// Reproduces PB-DP8's enumerated targeted-trigger `Complete` count (`>= 77`), exercising
/// the compound predicate (a `Triggered` node qualified by its OWN `targets` field)
/// against a known-good answer.
fn canonical_walk_reproduces_pb_dp8_roster() {
    let row = ROWS.iter().find(|r| r.id == "triggered_targets").unwrap();
    let count = all_cards()
        .iter()
        .filter(|d| is_effectively_complete(d))
        .filter(|d| (row.predicate)(&serde_json::to_value(d).unwrap()))
        .count();
    assert!(
        count >= 77,
        "triggered_targets has only {count} Complete defs, expected >= 77 (PB-DP8's own \
         enumerated number)"
    );
}

// ── T12: the row-key collision inventory (scoped to the plan's own cited cases) ─

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf()
}

fn read_ct(rel: &str) -> String {
    std::fs::read_to_string(workspace_root().join(rel))
        .unwrap_or_else(|e| panic!("cannot read {rel}: {e}"))
}

fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|line| match line.find("//") {
            Some(i) => &line[..i],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Counts lines whose TRIMMED content begins with `key` immediately followed by a
/// non-identifier character -- an enum variant DECLARATION shape (`Key,` / `Key {` /
/// `Key(..)`), not a usage site (`Foo::Key`, which never begins a trimmed line with the
/// bare `key` in these declaration-only files).
fn count_variant_declaration_lines(src: &str, key: &str) -> usize {
    strip_line_comments(src)
        .lines()
        .filter(|line| {
            let t = line.trim_start();
            t.strip_prefix(key)
                .and_then(|rest| rest.chars().next())
                .map(|c| !c.is_alphanumeric() && c != '_')
                .unwrap_or(false)
        })
        .count()
}

/// **Scoped inventory** (not a generic scan of all 22 row keys): the plan's own §4 note 4
/// / R5 name exactly these cases as needing a pinned reason. `Discover` is the one LIVE
/// collision (`Effect::Discover` struct variant + `KeywordAbility::Discover` unit
/// variant, accepted -- both reach the same auto-cast site per the keyword's own doc).
/// `SearchLibrary`/`Scry`/`Surveil` each have a "twin" declared in
/// `state/stubs.rs`'s `EffectChoiceQuestion` / `EffectChoiceAnswer` -- GameState-facing
/// wire types with NO structural path from `CardDefinition`, so they cannot inflate a
/// `CardDefinition`-serialization walk regardless of the name collision.
fn pinned_collision_counts() -> &'static [(&'static str, usize)] {
    &[
        ("Discover", 2),
        ("SearchLibrary", 3),
        ("Scry", 3),
        ("Surveil", 3),
    ]
}

#[test]
fn row_variant_name_collision_inventory_is_pinned() {
    let card_definition_src = read_ct("crates/card-types/src/cards/card_definition.rs");
    let types_src = read_ct("crates/card-types/src/state/types.rs");
    let stubs_src = read_ct("crates/card-types/src/state/stubs.rs");
    let combined = [&card_definition_src, &types_src, &stubs_src];

    for (key, expected) in pinned_collision_counts() {
        let total: usize = combined
            .iter()
            .map(|src| count_variant_declaration_lines(src, key))
            .sum();
        assert_eq!(
            total, *expected,
            "the number of enum-variant declarations named {key:?} across \
             card_definition.rs/types.rs/stubs.rs changed from the pinned {expected} to \
             {total} -- a new enum reusing this row's variant name would silently change \
             what the gate counts. If this is a deliberate new declaration, update the pin \
             AND re-argue (or re-scope) the row's predicate."
        );
    }
}

#[test]
fn collision_scan_is_not_vacuous() {
    let src = "    Discover,\n    Discover {\n    Discovering,\n";
    assert_eq!(
        count_variant_declaration_lines(src, "Discover"),
        2,
        "the scanner must count exactly the two declaration-shaped lines, not the \
         `Discovering,` line (which does not have a non-identifier char immediately after \
         `Discover`)"
    );
}

// ── T13: PROSE_FIELDS denylist completeness ────────────────────────────────────

/// A field declaration line's `(name, type)` if its type is exactly `String`,
/// `Option<String>` or `Vec<String>`.
fn string_field_name(line: &str) -> Option<String> {
    let t = line.trim();
    let t = t.strip_prefix("pub ").unwrap_or(t);
    let colon = t.find(':')?;
    let (name_part, type_part) = t.split_at(colon);
    let name = name_part.trim();
    if name.is_empty() || !name.chars().next().unwrap().is_lowercase() {
        return None;
    }
    let ty = type_part[1..].trim().trim_end_matches(',').trim();
    if ty == "String" || ty == "Option<String>" || ty == "Vec<String>" {
        Some(name.to_string())
    } else {
        None
    }
}

#[test]
/// Every `String` / `Option<String>` / `Vec<String>` field declared on `CardDefinition`,
/// `CardFace`, or the `AbilityDefinition` variants in `card_definition.rs`, plus
/// `TriggeredAbilityDef` in `game_object.rs` (reachable via `Effect::CreateEmblem`), must
/// be denylisted in `PROSE_FIELDS` -- a new prose field is a new false-positive channel
/// for the unit-variant rows.
fn prose_field_denylist_covers_every_string_field_in_the_dsl() {
    let mut found: BTreeSet<String> = BTreeSet::new();

    // The whole file for card_definition.rs: every struct/enum declared there
    // (CardDefinition, CardFace, the AbilityDefinition variants, TargetFilter, TokenSpec,
    // ...) is part of the CardDefinition DSL tree.
    let card_definition_src = read_ct("crates/card-types/src/cards/card_definition.rs");
    for line in strip_line_comments(&card_definition_src).lines() {
        if let Some(name) = string_field_name(line) {
            found.insert(name);
        }
    }

    // game_object.rs ALSO declares `Characteristics` (a runtime GameObject type, NOT
    // reachable from CardDefinition -- its `rules_text: String` is a false-positive risk
    // if the whole file were scanned). Scope to the `TriggeredAbilityDef` struct body only
    // (reachable via `Effect::CreateEmblem { triggered_abilities, .. }`).
    let game_object_src = read_ct("crates/card-types/src/state/game_object.rs");
    let stripped = strip_line_comments(&game_object_src);
    let start = stripped
        .find("pub struct TriggeredAbilityDef")
        .expect("TriggeredAbilityDef struct not found");
    let open = start
        + stripped[start..]
            .find('{')
            .expect("TriggeredAbilityDef: no opening brace");
    let bytes = stripped.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    let body_end = loop {
        assert!(i < bytes.len(), "TriggeredAbilityDef: unbalanced braces");
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    break i;
                }
            }
            _ => {}
        }
        i += 1;
    };
    for line in stripped[open + 1..body_end].lines() {
        if let Some(name) = string_field_name(line) {
            found.insert(name);
        }
    }
    assert!(
        found.len() >= 5,
        "the scan found only {} String-typed field names -- expected at least 5 \
         (name/oracle_text/subtype/prompt/has_name); the scan is probably not reaching the \
         corpus",
        found.len()
    );
    let missing: Vec<&String> = found
        .iter()
        .filter(|f| !PROSE_FIELDS.contains(&f.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "these String-typed fields are reachable from CardDefinition but NOT in \
         PROSE_FIELDS -- a false-positive channel for the unit-variant rows: {missing:?}"
    );
}

// ── T14: the SR-33 Gated rows do not drift from effect_choose_gate.rs ─────────

#[test]
/// The two `ROWS` entries classified `Gated` (`choose_stub` -> `Choose`,
/// `may_pay_or_else` -> `MayPayOrElse`) must name variants `effect_choose_gate.rs` still
/// actually bars. Scoped to those two rows -- the any-color mana-stub family
/// (`AddManaChoice`/`AddManaAnyColor*`) is also barred there but is deliberately NOT one
/// of the 22 §3.1 rows (plan §5.5: a different defect class, and merging them would
/// weaken SR-33's hard zero into a baseline).
fn sr33_gated_variants_are_represented_in_the_row_table() {
    let gate_src = read_ct("crates/engine/tests/core/effect_choose_gate.rs");
    let barred: BTreeSet<&str> = ["Choose", "MayPayOrElse", "AddManaChoice"]
        .into_iter()
        .filter(|key| gate_src.contains(&format!("def_uses(d, \"{key}\")")))
        .collect();
    assert!(
        barred.len() >= 2,
        "expected effect_choose_gate.rs to still bar at least Choose and MayPayOrElse via \
         def_uses(d, \"...\") -- got {barred:?}; the scan may be stale against a rename"
    );

    for id in ["choose_stub", "may_pay_or_else"] {
        let row = ROWS.iter().find(|r| r.id == id).unwrap();
        assert!(
            matches!(row.class, DecisionClass::Gated { .. }),
            "row {id:?} must be classified Gated"
        );
    }
    // The keys these two rows' predicates are built on must be within the barred set.
    assert!(
        barred.contains("Choose"),
        "choose_stub's key must still be barred"
    );
    assert!(
        barred.contains("MayPayOrElse"),
        "may_pay_or_else's key must still be barred"
    );
}

// ── T16: named residual seeds still exist in the audit ────────────────────────

#[test]
/// Every seed id named in any `DecisionClass::Served { residual }` must still appear in
/// `docs/audits/decision-point-audit.md` -- keeps the taxonomy's "served, with a residual"
/// claim honest in both directions: a row cannot claim a residual seed that has quietly
/// been removed from the audit (closed, renamed, or never filed).
fn named_residual_seed_ids_still_exist_in_the_audit() {
    let audit = read_ct("docs/audits/decision-point-audit.md");
    let mut checked = 0usize;
    for row in ROWS {
        if let DecisionClass::Served { residual, .. } = &row.class {
            for seed in *residual {
                checked += 1;
                assert!(
                    audit.contains(seed),
                    "row {:?} claims residual seed {seed:?}, which no longer appears in \
                     docs/audits/decision-point-audit.md",
                    row.id
                );
            }
        }
    }
    assert!(
        checked >= 1,
        "no residual seeds were checked -- the ROWS table's residual lists are all empty, \
         so this test is vacuous. If PB-DP9's residuals (OOS-DP9-9/OOS-DP9-3) were removed \
         from the search_library row, restore them or drop this test deliberately."
    );
}
