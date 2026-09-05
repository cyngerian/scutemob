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
//! **And it can only see a decision the DSL encoded.** Every row in `decision_site_walk.rs`'s
//! `ROWS` is a predicate over a *variant name*, so a card whose choice was dropped at
//! authoring time -- a "you may X. If you do, Y" written as a bare `Effect::Sequence`, a
//! "choose one" written as a single unconditional effect -- hits zero rows and passes T4/T6
//! forever. Smuggler's Copter is exactly this: CR 118.12's "you may draw a card. If you do,
//! discard a card" is authored as `Effect::Sequence(vec![DrawCards, DiscardCards])`, so the
//! `may` is gone and the only reason it appears in `BASELINE` at all is the incidental
//! `DiscardCards` in its second element. **That class is strictly worse than the class this
//! file records, and this file does not detect it** (OOS-DP10-9). Detecting it needs an
//! oracle-text-vs-DSL cross-check, a different instrument from a variant walk.
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
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};

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
        // PB-DX35: this row is `RevealAndRoute`-only now -- `LookAtTopThenPlace` has its
        // own row below, `look_at_top_then_place_optional`.
        "look_at_top_or_route" => serde_json::to_value(Effect::RevealAndRoute {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: TargetFilter::default(),
            matched_dest: ZoneTarget::Hand {
                owner: PlayerTarget::Controller,
            },
            unmatched_dest: ZoneTarget::Library {
                owner: PlayerTarget::Controller,
                position: LibraryPosition::Bottom,
            },
        }),
        "look_at_top_then_place_optional" => serde_json::to_value(Effect::LookAtTopThenPlace {
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
            optional: true,
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
///
/// **An entry asserts exactly one thing -- that this def hits these `AutoChosen` rows. It
/// asserts NOTHING about whether the def is otherwise oracle-correct.** The 2026-07-27
/// freeze populated this table mechanically from `T9`'s output, and was not cross-checked
/// against oracle text def-by-def.
///
/// **PB-DX4 (2026-08-01, `scutemob-168`) performed that triage and this table is now
/// oracle-read** -- all 97 frozen entries were read against MCP printed text and classified
/// plan §5.3 class-B (the def is faithful; the engine merely auto-picks among legal options,
/// which is the only thing an entry here claims) or class-D (the def is simply wrong). The
/// split was **86 B / 11 D**, so PB-DP10's 2-of-5 spot-check overstated the D rate roughly
/// fivefold -- the caution that "2-of-5 is a very noisy sample" was correct. Of the 11: five
/// were repaired in place and stayed `Complete` (Metastatic Evangel, Grisly Salvage, Satyr
/// Wayfinder, Sword of Truth and Justice, Radstorm); five were demoted and have therefore
/// LEFT this table (Smuggler's Copter -> `known_wrong`; Contaminant Grafter, Grateful
/// Apparition, Thrasios Triton Hero, Shambling Ghast -> `partial`); and one -- Staff of
/// Compleation's "target permanent you own" authored as `TargetController::You` -- was
/// deliberately left `Complete` and allowlisted in `completeness_deviation_scan.rs`, matching
/// the shipped `nether_traitor` decision for the identical owner-vs-controller class
/// (OOS-DX4-1 asks the corpus-wide question instead).
/// Per-def findings with oracle citations: `memory/primitives/pb-dx4-baseline-triage.md`.
///
/// **What that triage does NOT convert this table into.** An entry still asserts only its row
/// set. A def read as class-B on 2026-08-01 can drift afterwards, and nothing re-reads it --
/// the same dated-claim problem PB-DX3/PB-DX3b found in blocker notes. Read the triage doc's
/// date, not this table, when you need to know whether a def has been checked.
const BASELINE: &[(&str, &[&str], Option<&str>)] = &[
    ("Accursed Marauder", &["sacrifice_permanents"], None),
    ("Anowon, the Ruin Sage", &["sacrifice_permanents"], None),
    ("Atomize", &["proliferate"], None),
    ("Atraxa, Praetors' Voice", &["proliferate"], None),
    ("Blightbelly Rat", &["proliferate"], None),
    ("Bloated Contaminator", &["proliferate"], None),
    ("Bolt Bend", &["change_targets"], None),
    ("Brainstorm", &["put_on_library"], None),
    ("Butcher of Malakir", &["sacrifice_permanents"], None),
    ("Cached Defenses", &["bolster_amass"], None),
    ("Caged Sun", &["choose_color_or_type"], None),
    ("Cankerbloom", &["proliferate"], None),
    ("Chaos Warp", &["look_at_top_or_route"], None),
    ("Coiling Oracle", &["look_at_top_or_route"], None),
    ("Contagion Clasp", &["proliferate"], None),
    ("Contentious Plan", &["proliferate"], None),
    ("Crippling Fear", &["choose_color_or_type"], None),
    ("Deflecting Swat", &["change_targets"], None),
    ("Demon's Disciple", &["sacrifice_permanents"], None),
    ("Dictate of Erebos", &["sacrifice_permanents"], None),
    ("Dreadhorde Invasion", &["bolster_amass"], None),
    ("Dromoka, the Eternal", &["bolster_amass"], None),
    ("Drown in Ichor", &["proliferate"], None),
    ("Etchings of the Chosen", &["choose_color_or_type"], None),
    ("Evolution Sage", &["proliferate"], None),
    ("Felidar Retreat", &["modal_trigger"], None),
    ("Fleshbag Marauder", &["sacrifice_permanents"], None),
    ("Flusterstorm", &["counter_unless_pays"], None),
    ("Flux Channeler", &["proliferate"], None),
    ("Geological Appraiser", &["discover"], None),
    ("Goblin Ringleader", &["look_at_top_or_route"], None),
    ("Grave Pact", &["sacrifice_permanents"], None),
    ("Inexorable Tide", &["proliferate"], None),
    ("Izzet Charm", &["counter_unless_pays"], None),
    ("Karn's Bastion", &["proliferate"], None),
    ("Kindred Dominance", &["choose_color_or_type"], None),
    ("Korvold, Fae-Cursed King", &["sacrifice_permanents"], None),
    ("Make Disappear", &["counter_unless_pays"], None),
    ("Mana Leak", &["counter_unless_pays"], None),
    ("Mana Tithe", &["counter_unless_pays"], None),
    ("Merciless Executioner", &["sacrifice_permanents"], None),
    ("Metastatic Evangel", &["proliferate"], None),
    ("Misdirection", &["change_targets"], None),
    ("Morophon, the Boundless", &["choose_color_or_type"], None),
    ("Obelisk of Urd", &["choose_color_or_type"], None),
    ("Pact of the Serpent", &["choose_color_or_type"], None),
    ("Patchwork Banner", &["choose_color_or_type"], None),
    ("Radstorm", &["proliferate"], None),
    ("Raffine's Informant", &["connive"], None),
    ("Retreat to Kazandu", &["modal_trigger"], None),
    ("Roalesk, Apex Hybrid", &["proliferate"], None),
    ("Roiling Regrowth", &["sacrifice_permanents"], None),
    (
        "Shambling Ghast",
        &["modal_trigger"],
        Some(
            "PB-DX35 (2026-09, `OOS-DX4-2`): partial -> Complete. Its mode-1 target is now \
             scoped to mode 1 alone (`ModeSelection.mode_targets`), so `trigger_modal_plan` \
             picks the CR 700.2b-legal mode instead of removing the whole trigger -- but the \
             CONTROLLER still does not choose the mode (the same `modal_trigger` AutoChosen row \
             `Felidar Retreat` and `Retreat to Kazandu` already carry above).",
        ),
    ),
    ("Spell Pierce", &["counter_unless_pays"], None),
    ("Staff of Compleation", &["proliferate"], None),
    ("Stubborn Denial", &["counter_unless_pays"], None),
    ("Sword of Truth and Justice", &["proliferate"], None),
    ("Sylvan Messenger", &["look_at_top_or_route"], None),
    ("Tainted Observer", &["proliferate"], None),
    ("Tezzeret's Gambit", &["proliferate"], None),
    ("Thirsting Roots", &["proliferate"], None),
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
/// **PB-DX4 (2026-08-01, `scutemob-168`): lowered 97 -> 91**, and the six are named rather
/// than merely counted: Smuggler's Copter (-> `known_wrong`), Contaminant Grafter, Grateful
/// Apparition, Thrasios Triton Hero, Shambling Ghast and Hullbreaker Horror (-> `partial`)
/// were demoted by the
/// OOS-DP10-8 oracle-text triage of this very table, so they are no longer
/// effectively-`Complete` and drop out of the union. Measured by running `T6` and reading the
/// number it printed, not derived by arithmetic -- and that mattered THREE times: the first
/// reading was 93 (Shambling Ghast was still `Complete`), then 92 once its fourth defect --
/// the flat mode-target, OOS-DX4-2 -- was surfaced by fixing its first three, then 91 when the
/// closing review found `hullbreaker_horror` carrying that same flat-mode-target defect and
/// still `Complete`. A number that moved three times inside one batch is the argument for
/// reading it off `T6` rather than computing it. The six defs the same triage REPAIRED
/// (Metastatic Evangel, Grisly Salvage, Satyr Wayfinder, Sword of Truth and Justice, Radstorm,
/// Risen Reef) stayed `Complete` and none of them changed row set, so they contribute 0 here.
/// **ENG-1 (2026-08-02, `scutemob-191`): lowered 91 -> 80.** Not a demotion this time -- an
/// engine PB served the `discard_cards` row (`AutoChosen` -> `Served { by: "ENG-1" }`), so
/// every def whose *only* auto-chosen row was `discard_cards` drops out of the union entirely.
/// 11 of `BASELINE`'s 12 `discard_cards`-touching entries were exactly that (11 solo rows
/// deleted); `Izzet Charm` still hits `counter_unless_pays` and stays in the union, so the drop
/// is 11, not 12 -- read off `T6`'s printed number (80), not computed as `91 - 12`, per this
/// constant's own standing rule.
///
/// **PB-DX45 (2026-09-02, `scutemob-217`): lowered 80 -> 71.** The same shape as ENG-1's
/// move, on the next row over: CR 118.12's optional cost became a real player choice at
/// BOTH of the engine's `try_pay_optional_cost` call sites, so `may_pay_then_effect` moved
/// `AutoChosen` -> `Served { by: "PB-DX45" }`. Ten `Complete` defs hit that row; nine hit it
/// and nothing else and leave the union entirely, while `Tainted Observer` stays for
/// `proliferate` -- so the drop is 9, not 10, the same 1-off ENG-1 recorded for
/// `Izzet Charm`. **71 is read off `T6`'s printed number**, not computed as `80 - 9`, per
/// this constant's own standing rule; the arithmetic agreeing is a check, not the source.
///
/// **PB-DX35 Half A (2026-09, `OOS-DX4-2`): raised 71 -> 72.** `Shambling Ghast` flipped
/// `partial` -> `Complete` and hits `modal_trigger` (a NEW `BASELINE` entry above), the
/// same row `Felidar Retreat`/`Retreat to Kazandu` already carry -- one def added to the
/// union, read off `T6`'s printed number, not computed as `71 + 1`.
///
/// **PB-DX35 Half B (2026-09, `OOS-DX4-5`): lowered 72 -> 67.** The compound
/// `look_at_top_or_route` row split: `LookAtTopThenPlace`'s `optional` field became a
/// real CR 118.12 player decision and its half of the row moved to the new
/// `look_at_top_then_place_optional` Served row, which is the same shape as ENG-1's and
/// PB-DX45's moves on the rows either side of it. Five `BASELINE` entries removed
/// (`Birthing Ritual`, `Growing Rites of Itlimoc`, `Grisly Salvage`, `Satyr Wayfinder`,
/// `Risen Reef` -- none hits any OTHER AutoChosen row). `RevealAndRoute`'s CR 401.4
/// order choice stays behind on the split-off `look_at_top_or_route` row, AutoChosen,
/// unchanged (`Chaos Warp`/`Coiling Oracle`/`Goblin Ringleader`/`Sylvan Messenger` keep
/// their entries). Read off `T6`'s printed number, not computed as `72 - 5`, per this
/// constant's own standing rule.
const MAX_AUTO_CHOSEN_COMPLETE_UNION: usize = 67;

const MIN_ROWS: usize = 22;
const MIN_BASELINE: usize = 50;
const MIN_CORPUS: usize = 1000;

fn baseline_map() -> HashMap<&'static str, BTreeSet<&'static str>> {
    BASELINE
        .iter()
        .map(|(name, rows, _)| (*name, rows.iter().copied().collect()))
        .collect()
}

/// **The gate's own offender-detection logic**, extracted so both `T4` and its
/// non-vacuity probe (`t4_gate_logic_reddens_...`) drive the IDENTICAL code path (review
/// finding PB-DP10 #3 -- the probe previously re-checked two predicates T1/T5 already
/// cover and never executed this loop at all). Every effectively-`Complete` def hitting
/// one or more `AutoChosen` rows must either be absent from `baseline` (offender) or
/// present with an EXACT row-set match. A mismatch is also an offender: a superset of the
/// recorded rows means the def gained a decision since the freeze, and a subset means the
/// entry should be tightened.
fn offenders(
    defs: &[mtg_engine::CardDefinition],
    baseline: &HashMap<&str, BTreeSet<&'static str>>,
) -> Vec<String> {
    let mut offenders: Vec<String> = Vec::new();

    for def in defs {
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

    offenders
}

/// `T4`'s failure message, extracted so `t4_failure_message_names_the_bound` (review
/// finding PB-DP10 #5) can assert against it directly instead of the module doc citing a
/// test that does not exist.
fn t4_message(offenders: &[String]) -> String {
    format!(
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
            reason -- a recorded acknowledgement that this card ships with the engine \
            choosing for the player until the owning PB lands.\n\n\
         Implementing the choice properly is NOT an exit for this batch: it needs the \
         owning engine PB (docs/audits/decision-point-audit.md §5, rows DP-13..DP-31), \
         not a card-def edit.\n\nOffenders:\n{}",
        offenders.join("\n")
    )
}

#[test]
/// **The gate.** See [`offenders`] and [`t4_message`] for what this asserts.
fn no_complete_def_introduces_an_unrecorded_auto_chosen_decision() {
    let baseline = baseline_map();
    let defs = all_cards();
    let found = offenders(&defs, &baseline);
    assert!(found.is_empty(), "{}", t4_message(&found));
}

#[test]
/// `T4`'s own module doc claims a machine check against the R6 harm scenario ("the gate
/// reading as a closure of DP-INV") by naming this test. Review finding PB-DP10 #5: the
/// test did not exist. Written now: [`t4_message`]'s text must contain the four load-bearing
/// phrases a reader needs to not mistake this gate for a closure -- the CANNOT-STOP-THE-
/// GROWTH bound itself, and both of the two (and only two) legal exits.
fn t4_failure_message_names_the_bound() {
    let msg = t4_message(&[
        "Fake Offender is NOT in BASELINE but hits {\"proliferate\"}. \
                            Fake Offender hits proliferate (CR 701.34a, effects/mod.rs)"
            .to_string(),
    ]);
    for phrase in [
        "CANNOT STOP THE GROWTH",
        "Mark the def non-Complete",
        "Add a BASELINE entry",
        "is NOT an exit for this batch",
    ] {
        assert!(
            msg.contains(phrase),
            "T4's failure message must contain {phrase:?} so a reader cannot mistake this \
             gate for a closure of DP-INV (plan §12 R6); got:\n{msg}"
        );
    }
}

#[test]
/// T4's gate logic is not vacuously green: this drives the SAME [`offenders`] function T4
/// calls, against a synthetic three-def corpus, never touching `all_cards()`. Review
/// finding PB-DP10 #3: the original probe re-checked two predicates T1/T5 already cover
/// and never executed the offender loop at all -- in particular the `Some(recorded) if
/// recorded != &hits` mismatch arm (half the ratchet's design rationale, plan §1.3) had NO
/// coverage anywhere. This probe exercises all three outcomes the review named.
fn t4_gate_logic_reddens_on_a_new_unbaselined_auto_chosen_complete_def() {
    fn prolif_def(name: &str) -> mtg_engine::CardDefinition {
        mtg_engine::CardDefinition {
            name: name.to_string(),
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
        }
    }

    // (a) An unbaselined Complete Proliferate def IS an offender.
    let mut unbaselined = prolif_def("PB-DP10 Synthetic Offender (unbaselined)");
    unbaselined.completeness = mtg_engine::cards::Completeness::Complete;

    // (b) A def present in a synthetic baseline with a SMALLER recorded row set than it
    // actually hits IS an offender, on the "tighten the entry" (subset) arm -- the
    // previously-uncovered mismatch branch.
    let mut mismatched = prolif_def("PB-DP10 Synthetic Offender (mismatched baseline)");
    mismatched.completeness = mtg_engine::cards::Completeness::Complete;

    // (c) A NON-Complete def carrying the identical site is NOT an offender.
    let mut non_complete = prolif_def("PB-DP10 Synthetic Non-Offender (not Complete)");
    non_complete.completeness = mtg_engine::cards::Completeness::partial("T4 probe, not real");

    // ENG-1 (2026-08-02): `discard_cards` flipped AutoChosen -> Served, so a synthetic def
    // that only hits `proliferate` can never actually hit it -- `auto_chosen_row_hits` filters
    // to AutoChosen rows before matching, so a stale "discard_cards" here would still pass
    // (the recorded set is still a strict superset of the actual hits) but would demonstrate
    // the subset/tighten arm with a row id no def can ever hit anymore, which is a weaker
    // probe. Swapped for `sacrifice_permanents`, still live AutoChosen.
    let baseline: HashMap<&str, BTreeSet<&'static str>> = [(
        mismatched.name.as_str(),
        ["proliferate", "sacrifice_permanents"]
            .into_iter()
            .collect(),
    )]
    .into_iter()
    .collect();
    assert!(
        !baseline.contains_key(unbaselined.name.as_str()),
        "test setup: the unbaselined synthetic name must not collide"
    );

    let corpus = [
        unbaselined.clone(),
        mismatched.clone(),
        non_complete.clone(),
    ];
    let found = offenders(&corpus, &baseline);

    assert!(
        found
            .iter()
            .any(|o| o.contains(unbaselined.name.as_str()) && o.contains("is NOT in BASELINE")),
        "(a) an unbaselined Complete Proliferate def must be an offender: {found:?}"
    );
    assert!(
        found
            .iter()
            .any(|o| o.contains(mismatched.name.as_str()) && o.contains("tighten the entry")),
        "(b) a def baselined with a recorded row set that is a SUPERSET of its actual hits \
         (the subset/tighten mismatch arm) must be an offender: {found:?}"
    );
    assert!(
        !found.iter().any(|o| o.contains(non_complete.name.as_str())),
        "(c) a non-Complete def must never be an offender, even carrying the identical \
         site: {found:?}"
    );
    assert_eq!(
        found.len(),
        2,
        "exactly two of the three synthetic defs are offenders: {found:?}"
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
                 recorded acknowledgement needs a real sentence, not a stub",
                reason_text.len()
            );
        }
    }

    // The `Some(reason)` half of T4's message is a CONTRACT, so it needs an enforcer.
    //
    // T4's failure message tells an author that exit 2 is "add a `BASELINE` entry with the
    // def's exact row set **and a written reason**". The `if let Some(..)` above only
    // validates a reason that is already there, so without this check an author could append
    // `("New Card", &["proliferate"], None)` and go green with no justification at all --
    // and, worse, the new entry would be indistinguishable from the 2026-07-27 freeze, which
    // is the ONLY thing `None` is documented to mean. (Found by the closing `/review`, which
    // also noted the mitigation: T6's exact-equality ratchet forces a same-commit
    // `MAX_AUTO_CHOSEN_COMPLETE_UNION` bump, so the addition could not have been *silent* --
    // but "not silent" is a weaker property than "justified", and the message promises the
    // stronger one. A message that states a requirement nothing enforces is the OOS-DP7-11
    // class: a claim wearing a gate's authority.)
    //
    // The freeze is closed at exactly its measured size, so every LATER entry must carry
    // `Some(reason)`. This deliberately does not grow: shrinking is fine (a def demoted or
    // fixed just leaves), growing is not.
    // PB-DX4 fix cycle (2026-08-01, `scutemob-168`, review Finding 3): **92, not 97.**
    //
    // This ceiling is the enforcer for T4's promise that a post-freeze entry carries a written
    // reason. PB-DX4 removed five frozen entries (the five defs it demoted), which left the
    // ceiling five above the population it bounds — so the gate would have accepted five NEW
    // `None` entries with no justification, silently, which is precisely what the comment
    // below says it must not do ("this deliberately does not grow"). Lowering it in the same
    // commit as the removals is what keeps the promise true. Derivation: 97 frozen entries
    // minus `smugglers_copter`, `contaminant_grafter`, `grateful_apparition`,
    // `thrasios_triton_hero`, `shambling_ghast` = 92, equal to `BASELINE.len()` because every
    // surviving entry is still a freeze entry (this batch added none).
    const FROZEN_2026_07_27: usize = 91;
    let unexplained = BASELINE.iter().filter(|(_, _, r)| r.is_none()).count();
    assert!(
        unexplained <= FROZEN_2026_07_27,
        "{unexplained} BASELINE entries carry no written reason, but only the \
         {FROZEN_2026_07_27} surviving entries of the 2026-07-27 PB-DP10 freeze are allowed \
         to. Every \
         entry added after that freeze is a deliberate act and must carry `Some(reason)` \
         naming why this card ships with the engine choosing for the player -- that is what \
         `no_complete_def_introduces_an_unrecorded_auto_chosen_decision`'s failure message \
         promises an author, and this is where the promise is kept."
    );
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
    // Review finding PB-DP10 #14: serialize each def ONCE, not once per row (~1,804 vs
    // ~3,600 serializations here -- the pattern is worse in T9, which is the same fix
    // applied at scale).
    let jsons: Vec<serde_json::Value> = defs
        .iter()
        .map(|d| serde_json::to_value(d).unwrap())
        .collect();
    for id in ["add_mana_filter_choice", "the_ring_tempts_you"] {
        let row = ROWS.iter().find(|r| r.id == id).unwrap();
        let complete: Vec<String> = defs
            .iter()
            .zip(&jsons)
            .filter(|(d, _)| is_effectively_complete(d))
            .filter(|(_, json)| (row.predicate)(json))
            .map(|(d, _)| d.name.clone())
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

    // Each Served row's roster floor is non-zero. Serialize once (Finding PB-DP10 #14),
    // reused across the 5 rows checked below.
    let defs = all_cards();
    let jsons: Vec<serde_json::Value> = defs
        .iter()
        .map(|d| serde_json::to_value(d).unwrap())
        .collect();
    for (id, min) in [
        ("triggered_targets", 1usize),
        ("search_library", 1),
        ("scry", 1),
        ("surveil", 1),
        // ENG-1 (2026-08-02): `discard_cards` flipped AutoChosen -> Served. Same
        // non-zero-floor treatment as its three siblings above.
        ("discard_cards", 1),
        // PB-DX35 (2026-09, `OOS-DX4-5`): `LookAtTopThenPlace`'s optional placement is
        // now a real CR 118.12 choice, and the row split off `look_at_top_or_route` (see
        // that row's own updated site string) rather than merely gained a residual note.
        ("look_at_top_then_place_optional", 1),
    ] {
        let row = ROWS.iter().find(|r| r.id == id).unwrap();
        assert!(
            matches!(row.class, DecisionClass::Served { .. }),
            "row {id:?} must be classified Served"
        );
        let count = defs
            .iter()
            .zip(&jsons)
            .filter(|(d, _)| is_effectively_complete(d))
            .filter(|(_, json)| (row.predicate)(json))
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
    // Review finding PB-DP10 #14: this test was the worst offender -- 22 rows x ~1,804
    // defs re-serialized `CardDefinition` ~40,000 times where 1,804 does. Serialize once,
    // index by position for every row below.
    let jsons: Vec<serde_json::Value> = defs
        .iter()
        .map(|d| serde_json::to_value(d).unwrap())
        .collect();
    println!("PB-DP10 decision-site reconciliation (enumerated from all_cards(), not grep):");
    for row in ROWS {
        let mut complete = 0usize;
        let mut other = 0usize;
        for (def, json) in defs.iter().zip(&jsons) {
            if (row.predicate)(json) {
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
    // Hoisted per Finding PB-DP10 #14: serialize once, reused across the 3 rows below.
    let jsons: Vec<serde_json::Value> = defs
        .iter()
        .map(|d| serde_json::to_value(d).unwrap())
        .collect();
    // PB-DX4 (2026-08-01, `scutemob-168`): `scry` lowered 16 -> 15. Legitimate corpus
    // movement, not detector drift, and the same shape as PB-DX3b's 77 -> 76 on the PB-DP8
    // roster below: `thrasios_triton_hero` carries an `Effect::Scry` inside the activated
    // ability whose OTHER half -- printed "Otherwise, draw a card", authored as
    // `RevealAndRoute`'s `unmatched_dest: ZoneTarget::Hand`, a zone move that fires no draw
    // event -- made it class-D in the OOS-DP10-8 triage, so it is now `partial` and no longer
    // counts as `Complete`. Its scry half was always correct; it leaves this count for a
    // reason that has nothing to do with scry. `search_library` and `surveil` are unmoved
    // (verified by running this test, not assumed: only the scry arm reddened).
    for (id, floor) in [("search_library", 73usize), ("scry", 15), ("surveil", 8)] {
        let row = ROWS.iter().find(|r| r.id == id).unwrap();
        let count = defs
            .iter()
            .zip(&jsons)
            .filter(|(d, _)| is_effectively_complete(d))
            .filter(|(_, json)| (row.predicate)(json))
            .count();
        assert!(
            count >= floor,
            "row {id:?} has only {count} Complete defs, expected >= {floor} (PB-DP9's own \
             published number)"
        );
    }
}

/// A byte-faithful replica of `primitives/pb_dp9_effect_choice.rs::roster`'s walk: it
/// matches OBJECT KEYS ONLY and is therefore blind to a unit variant, which serializes as
/// a bare JSON string. This is deliberately the *weaker* of the two walks — it exists so
/// [`pb_dp9_roster_walks_agree_by_value`] can compare the two by value from inside a single
/// test target.
///
/// **Do not "fix" this to call [`crate::decision_site_walk::json_contains_variant`].** Making the
/// replica correct would make the equality check below trivially true and delete the only
/// mechanical link between the copy and the canonical walk.
fn key_only_contains_variant(v: &serde_json::Value, variant: &str) -> bool {
    match v {
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(k, child)| k == variant || key_only_contains_variant(child, variant)),
        serde_json::Value::Array(items) => {
            items.iter().any(|c| key_only_contains_variant(c, variant))
        }
        _ => false,
    }
}

#[test]
/// **PB-DX7 (OOS-DP10-1 rider).** The seed's stated mitigation for keeping a second copy of
/// the decision-site walk in `primitives/pb_dp9_effect_choice.rs` is that the copy is
/// "cross-checked BY VALUE (not by text) against the canonical walk". Re-verified at HEAD:
/// the canonical walk is byte-unchanged since PB-DP10's review-fix commit (`0d4adcb5`) —
/// the two later edits to `decision_site_walk.rs` (`87594d08` ENG-1, `cf89a213` PB-DX25c)
/// moved the `discard_cards` and `change_targets` ROW metadata only, neither of which is
/// one of PB-DP9's three rows — and both walks return **74 / 15 / 8** today.
///
/// But `canonical_walk_reproduces_pb_dp9_rosters` above asserts **floors** (73 / 15 / 8),
/// not agreement, and `search_library`'s floor sits one below the live count. A one-def
/// divergence between the copy and the canonical walk would therefore pass both tests in
/// silence — the cross-check was checking that each walk was individually plausible, not
/// that they agreed. This test closes that: for each of PB-DP9's three targets, the
/// key-only walk (the copy's algorithm) and the unit-variant-aware canonical walk must
/// return the **same** count.
///
/// The `..._DISAGREE` half is what makes the equality half mean something. Without it,
/// this test would pass just as well if `key_only_contains_variant` had been silently
/// replaced by a call to the canonical walk. `Effect::Proliferate` and
/// `Effect::TheRingTemptsYou` are unit variants (`card_definition.rs:1933`/`:2122`), so
/// they serialize as bare strings that only the canonical walk can see — the exact blindness
/// OOS-DP10-1 names. If a future refactor makes these two agree, the equality half above has
/// lost its teeth and the seed's residual hazard must be re-assessed, not the assertion
/// relaxed.
///
/// **Residual, stated plainly (`/review` L6, 2026-08-12):** this test compares the canonical
/// walk against `key_only_contains_variant` — a REPLICA of `primitives/pb_dp9_effect_choice
/// .rs::roster`'s algorithm, hand-copied into this file (see its own doc above: "a
/// byte-faithful replica"). It does NOT read the real copy. What this test proves is that the
/// ALGORITHM `key_only_contains_variant` implements is blind to unit variants (with a real
/// discriminating control, `Proliferate` 23 vs 0) — it does not prove the real copy in
/// `pb_dp9_effect_choice.rs` still IS that algorithm today. If the real copy drifts away from
/// the replica (a rename, a refactor, a bug fix applied to one but not the other), this test
/// stays green throughout, because it never touches the real copy at all. The replica was
/// verified byte-faithful to the real copy AT THE TIME this test was written
/// (`pb-DX7-execution-notes.md` §4.1); nothing re-verifies that afterward. The durable fix —
/// promoting the walk to one shared function both files call — is out of scope here (an engine
/// change) and is recorded as such, not silently deferred.
fn pb_dp9_roster_walks_agree_by_value() {
    let defs = all_cards();
    let jsons: Vec<serde_json::Value> = defs
        .iter()
        .map(|d| serde_json::to_value(d).unwrap())
        .collect();

    let count = |pred: &dyn Fn(&serde_json::Value) -> bool| -> usize {
        defs.iter()
            .zip(&jsons)
            .filter(|(d, _)| is_effectively_complete(d))
            .filter(|(_, j)| pred(j))
            .count()
    };

    let mut checked = 0usize;
    for variant in ["SearchLibrary", "Scry", "Surveil"] {
        let canonical = count(&|j| crate::decision_site_walk::json_contains_variant(j, variant));
        let key_only = count(&|j| key_only_contains_variant(j, variant));
        println!("  {variant}: canonical={canonical} key_only={key_only}");
        assert_eq!(
            canonical, key_only,
            "OOS-DP10-1: the pb_dp9_effect_choice.rs roster copy (key-only walk) and the \
             canonical unit-variant-aware walk disagree on {variant} ({key_only} vs \
             {canonical}). The copy is kept only because it is cross-checked by value \
             against the canonical walk; that cross-check has just failed, so either the \
             copy has drifted or {variant} has acquired a unit-variant spelling."
        );
        assert!(
            canonical > 0,
            "{variant} roster collapsed to zero — both walks are vacuous, so their \
             agreement proves nothing"
        );
        checked += canonical;
    }
    assert!(
        checked >= 60,
        "only {checked} defs matched across all three PB-DP9 targets; the walks are \
         under-counting and the equality above is near-vacuous"
    );

    // Discriminating control: the two walks MUST disagree on a unit variant, or the
    // equality assertions above are not testing anything.
    //
    // `Proliferate` only. `Effect::TheRingTemptsYou` is the other unit variant OOS-DP10-1's
    // note names, and it was tried here first — it is carried by **0** `Complete` defs in the
    // corpus today (measured, not assumed: both walks return 0, and the control reddened on
    // its own first run). A control that compares 0 to 0 discriminates nothing, so it is
    // stated here rather than left in as a passing-looking assertion. `Proliferate` measures
    // canonical=23 / key_only=0, which is the blindness the seed describes.
    let unit_variant = "Proliferate";
    let canonical = count(&|j| crate::decision_site_walk::json_contains_variant(j, unit_variant));
    let key_only = count(&|j| key_only_contains_variant(j, unit_variant));
    println!("  (control) {unit_variant}: canonical={canonical} key_only={key_only}");
    assert!(
        canonical > key_only,
        "control failed: the key-only walk found {key_only} defs carrying the UNIT variant \
         {unit_variant} and the canonical walk found {canonical}. The key-only replica is \
         supposed to be blind to unit variants (that IS OOS-DP10-1); if it is not, \
         `key_only_contains_variant` has been replaced by the canonical walk and the equality \
         assertions above are trivially true."
    );
}

#[test]
/// Reproduces PB-DP8's enumerated targeted-trigger `Complete` count. Originally pinned
/// at `>= 77`; **lowered to `>= 76` by PB-DX3b (2026-08-01)**, and the reason is
/// legitimate corpus movement, not detector drift: `emeria_the_sky_ruin` had been
/// counted in PB-DP8's 77 only because it was `Complete` by the
/// `#[default] Completeness::Complete` derive trap (nobody had ever set an explicit
/// marker on it), and its `AbilityDefinition::Triggered` upkeep ability DOES carry a
/// `targets: vec![TargetRequirement::TargetCardInYourGraveyard(..)]` (so it matched
/// this row's predicate). PB-DX3b demoted it to an explicit `Completeness::partial(..)`
/// because its printed "you may return" clause is genuinely unimplemented (see
/// `emeria_the_sky_ruin.rs`'s completeness note) — a correction, not a regression, of
/// a count that was never actually verified card-by-card in the first place. Measured
/// directly against `all_cards()` on this branch: `count == 76`.
///
/// **PB-DX4 (2026-08-01, `scutemob-168`): 76 -> 75**, and for the same reason a second time.
/// `shambling_ghast` counted here because its modal `WhenDies` trigger carries
/// `targets: vec![TargetRequirement::TargetCreatureWithFilter(..)]`; PB-DX4's OOS-DP10-8
/// oracle triage demoted it to an explicit `partial`. Note WHICH defect the marker is for:
/// the three deviations the triage went looking for (a phantom `Decayed` keyword, a permanent
/// `MinusOneMinusOne` counter for a printed "until end of turn", a stored `oracle_text` naming
/// "enters" against a `WhenDies` trigger) were all FIXED in place. The marker is for a fourth
/// the fix surfaced -- the very `targets` field this row counts is declared flat rather than
/// per-mode, so it is required whichever mode is chosen and a Ghast dying while no opponent
/// controls a creature produces nothing at all (CR 603.3d). Measured directly against
/// `all_cards()` on this branch: `count == 75`.
fn canonical_walk_reproduces_pb_dp8_roster() {
    let row = ROWS.iter().find(|r| r.id == "triggered_targets").unwrap();
    let defs = all_cards();
    let count = defs
        .iter()
        .filter(|d| is_effectively_complete(d))
        .filter(|d| (row.predicate)(&serde_json::to_value(d).unwrap()))
        .count();
    assert!(
        count >= 59,
        "triggered_targets has only {count} Complete defs, expected >= 59 (was 74 after \
         PB-DX3b's -1 and PB-DX4's -2; re-pinned DOWN by PB-DX28 §1 -- the 10 Karoos, \
         shrieking_drake, whitemane_lion and sword_of_truth_and_justice's AddCounter trigger \
         were migrated OFF a declared `TargetRequirement` onto `EffectTarget::ChosenObject` \
         (CR 115.10: none of the seven printed clauses says \"target\"), so their Triggered \
         abilities no longer carry a non-empty `targets` list and this predicate correctly \
         stops counting them; re-pinned DOWN AGAIN by PB-DX35 (2026-09, `OOS-DX4-2`) -- \
         `retreat_to_kazandu` (already `Complete`) had its mode-0 target re-shaped OFF the \
         flat `targets` list and into `ModeSelection.mode_targets`, so its Triggered ability \
         no longer carries a non-empty flat `targets` list either. `shambling_ghast` (this \
         batch's OTHER re-shape) contributes NOTHING to the move: it was excluded from 60 by \
         `Completeness::partial` and is excluded from 59 by its own now-empty flat `targets` \
         -- excluded both times, for two different reasons. 59 is the MEASURED count at this \
         batch's HEAD, not back-derived arithmetically)"
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

/// Strips `/* ... */` block comments (PB-DX32 fix cycle, review finding M8).
/// `strip_line_comments` above only truncates at `//`, per line -- it knows nothing
/// about `/* ... */`, so a `UNOBSERVABLE_ROW_IDS` tuple wrapped in a block comment
/// compiled out of the roster (the compiler drops it, `ROW_COUNT` shrinks) while
/// `quoted_strings` still found both of its string literals INSIDE the comment text,
/// leaving `runtime_decision_coverage_roster_matches_rows` green against a silently
/// shrunk roster. Naive (does not understand string literals containing `/*`), same
/// as `strip_line_comments`'s own naivety about `//` inside a string -- adequate for
/// this data file, not a general Rust tokenizer.
fn strip_block_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut rest = src;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find("*/") {
            Some(end) => rest = &after[end + 2..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
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
/// `CardDefinition`-serialization walk regardless of the name collision. **`ChooseColor`
/// / `ChooseCreatureType`** (review finding PB-DP10 #11) are the ONE row whose predicate
/// spans two enums BY DESIGN: `state/replacement_effect.rs`'s `ReplacementModification`
/// declares `ChooseColor(Color)` and `ChooseCreatureType(SubType)` (the as-enters, CR
/// 614.12a path), and `cards/card_definition.rs`'s `Effect` separately declares
/// `ChooseCreatureType { default: SubType }` (the resolution-time, CR 608.2d path) --
/// this is the row MOST exposed to a new declaration silently changing what the gate
/// counts, and until this fix it was the only one left unguarded.
fn pinned_collision_counts() -> &'static [(&'static str, usize)] {
    &[
        ("Discover", 2),
        ("SearchLibrary", 3),
        ("Scry", 3),
        ("Surveil", 3),
        ("ChooseColor", 1),
        ("ChooseCreatureType", 2),
    ]
}

#[test]
fn row_variant_name_collision_inventory_is_pinned() {
    let card_definition_src = read_ct("crates/card-types/src/cards/card_definition.rs");
    let types_src = read_ct("crates/card-types/src/state/types.rs");
    let stubs_src = read_ct("crates/card-types/src/state/stubs.rs");
    let replacement_effect_src = read_ct("crates/card-types/src/state/replacement_effect.rs");
    let combined = [
        &card_definition_src,
        &types_src,
        &stubs_src,
        &replacement_effect_src,
    ];

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
/// `Option<String>`, `Vec<String>`, or one of the two NEWTYPE-over-`String` types in
/// `crates/card-types/src` (`SubType(pub String)`, `CardId(pub String)`) in any of their
/// bare/`Option`/`Vec`/`Option<Vec<_>>` forms.
///
/// Review finding PB-DP10 #6: the original scan recognized only the three literal
/// `String` shapes, but serde serializes a single-field newtype STRUCT transparently --
/// `SubType("Human".to_string())` serializes to the bare JSON string `"Human"`, exactly
/// like a literal `String` field would -- so a `SubType`/`CardId`-typed field is an
/// EQUALLY real false-positive channel for the unit-variant rows and was silently
/// excluded from the completeness proof this function backs.
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
    if matches!(
        ty,
        "String"
            | "Option<String>"
            | "Vec<String>"
            | "SubType"
            | "Option<SubType>"
            | "Vec<SubType>"
            | "Option<Vec<SubType>>"
            | "CardId"
            | "Option<CardId>"
    ) {
        Some(name.to_string())
    } else {
        None
    }
}

#[test]
/// Every `String` / `Option<String>` / `Vec<String>` field, AND every field typed as one
/// of the two newtype-over-`String` types (`SubType`, `CardId`, in their bare/`Option`/
/// `Vec` forms -- review finding PB-DP10 #6), declared on `CardDefinition`, `CardFace`,
/// or the `AbilityDefinition` variants in `card_definition.rs`; `TriggeredAbilityDef` in
/// `game_object.rs` (reachable via `Effect::CreateEmblem`); and the three further files
/// that contribute types to the `CardDefinition` tree -- `state/types.rs`,
/// `state/replacement_effect.rs`, `state/targeting.rs` -- must be denylisted in
/// `PROSE_FIELDS`: a new prose field, or a new `SubType`/`CardId` field, is an equally
/// real false-positive channel for the unit-variant rows.
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

    // Review finding PB-DP10 #6: these three files contribute types reachable from
    // `CardDefinition` (`state/types.rs` supplies `TargetFilter` and the Enchant
    // restriction shape; `state/replacement_effect.rs` supplies `ReplacementModification`,
    // reachable via `AbilityDefinition::Replacement { modification, .. }`;
    // `state/targeting.rs` supplies target-request types) and were not scanned before
    // this fix -- scanned whole-file, same as card_definition.rs above.
    for rel in [
        "crates/card-types/src/state/types.rs",
        "crates/card-types/src/state/replacement_effect.rs",
        "crates/card-types/src/state/targeting.rs",
    ] {
        let src = read_ct(rel);
        for line in strip_line_comments(&src).lines() {
            if let Some(name) = string_field_name(line) {
                found.insert(name);
            }
        }
    }

    // game_object.rs ALSO declares `Characteristics` (a runtime GameObject type, NOT
    // reachable from CardDefinition -- its `rules_text: String` is a false-positive risk
    // if the whole file were scanned). Scope to the `TriggeredAbilityDef` struct body only
    // (reachable via `Effect::CreateEmblem { triggered_abilities, .. }`).
    let game_object_src = read_ct("crates/card-types/src/state/game_object.rs");
    let stripped = strip_line_comments(&game_object_src);
    let start = stripped
        .find("pub struct TriggeredAbilityDef {")
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

// ── T15: DELIBERATELY NOT BUILT ────────────────────────────────────────────────
//
// The numbering skips 15 on purpose, so a reader of this file alone is not left wondering.
// The plan's T15 was a roster digest (count + blake3 of the sorted variant-name lists of
// `Effect` / `AbilityDefinition` / `ReplacementModification`) that would redden when a new
// variant landed, with the message "classify it in `decision_site_walk.rs::ROWS`". It was
// dropped for budget under the plan's own §12 R9 ranking, and the drop is argued rather than
// merely admitted: a new variant of any of those three ALREADY forces a PROTOCOL and a HASH
// bump, because all three are inside the SR-8 / SR-17 wire closures -- so the *notice* was
// never the missing half. The missing half is the *obligation* to classify it here, which is
// what the digest would have supplied. Tracked as **OOS-DP10-11**; its `GameEvent` sibling is
// **OOS-DP10-7**. Audit §10's ledger counts both as feasible-but-not-built, not mechanized.

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

// ── T17: decision-point RUNTIME coverage roster matches ROWS (PB-DX32 Stage 6) ────
//
// `crates/simulator/src/decision_coverage.rs` carries a SEPARATE id-only roster (no
// predicates, no CR cites, no classification logic -- those stay here, exactly once)
// so `crates/simulator` can fold a runtime observation count without either crate
// dev-depending on the other's test tree. This is the source gate that keeps the two
// rosters from drifting: it EXTENDS the static gate (reads the simulator file as
// text), it does NOT rebuild it -- `ROWS`, `BASELINE` and
// `MAX_AUTO_CHOSEN_COMPLETE_UNION` above are untouched.

/// Every quoted string literal in `src`, in source order. An unescaped backslash
/// consumes the following character too (so it can never itself close the string,
/// which is all this needs: line-continuation escapes and any hypothetical `\"` both
/// stay inside the literal rather than ending it early).
fn quoted_strings(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '"' {
            continue;
        }
        let mut s = String::new();
        while let Some(next) = chars.next() {
            if next == '\\' {
                s.push(next);
                if let Some(escaped) = chars.next() {
                    s.push(escaped);
                }
                continue;
            }
            if next == '"' {
                break;
            }
            s.push(next);
        }
        out.push(s);
    }
    out
}

/// The text of one `pub const <NAME>: ... = &[ ... ];` array literal, from the const's
/// declaration to its OWN terminating `];` -- block-scoped, so `rustfmt` re-wrapping
/// the array cannot defeat it (mirrors this file's own `read_ct`/`strip_line_comments`
/// idiom, F17 of the PB-DX32 plan). `anchor` must be unambiguous against a
/// LONGER name sharing the same suffix (e.g. `"const OBSERVABLE_ROW_IDS"` does NOT
/// match inside `"const UNOBSERVABLE_ROW_IDS"`, because `"UN"` sits between `"const "`
/// and `"OBSERVABLE"` there).
fn extract_const_array_block<'a>(src: &'a str, anchor: &str) -> &'a str {
    let start = src
        .find(anchor)
        .unwrap_or_else(|| panic!("{anchor:?} not found in decision_coverage.rs"));
    let after = &src[start..];
    let end = after
        .find("];")
        .unwrap_or_else(|| panic!("no terminating '];' found after {anchor:?}"));
    &after[..end + 2]
}

/// **T17 (PB-DX32 Stage 6)** — `crates/simulator/src/decision_coverage.rs`'s
/// `OBSERVABLE_ROW_IDS` ∪ `UNOBSERVABLE_ROW_IDS` must equal `ROWS`'s ids exactly, and
/// `OBSERVABLE_ROW_IDS` must equal exactly the ids whose class is `Served`. Comments
/// are stripped FIRST — both `//` (`strip_line_comments`) and `/* ... */`
/// (`strip_block_comments`, PB-DX32 fix cycle, review finding M8) — so a row moved
/// into EITHER comment form is NOT counted as present. Also asserts the raw (pre-set)
/// id COUNT against `ROWS.len()`, which the set comparisons below cannot: a
/// duplicated id collapses invisibly inside a `BTreeSet`, so without this a
/// duplicate-plus-drop pair could cancel out and still pass. This is the
/// comment-satisfiable-gate class PB-DX22's review cycle 2 found in this exact
/// family (see this file's own header note) — closed here a second time, for block
/// comments.
#[test]
fn runtime_decision_coverage_roster_matches_rows() {
    let src = strip_block_comments(&strip_line_comments(&read_ct(
        "crates/simulator/src/decision_coverage.rs",
    )));

    let observable_block = extract_const_array_block(&src, "const OBSERVABLE_ROW_IDS");
    let unobservable_block = extract_const_array_block(&src, "const UNOBSERVABLE_ROW_IDS");

    let observable_raw = quoted_strings(observable_block);
    let observable: BTreeSet<String> = observable_raw.iter().cloned().collect();

    let unobservable_all = quoted_strings(unobservable_block);
    assert_eq!(
        unobservable_all.len() % 2,
        0,
        "UNOBSERVABLE_ROW_IDS must be a flat list of (id, reason) string-literal pairs \
         -- found an ODD number of quoted strings ({}), so a tuple is malformed or the \
         block boundary was mis-detected",
        unobservable_all.len()
    );
    // Every (id, reason) tuple's FIRST string is the id; the reason is never compared
    // against ROWS ids.
    let unobservable: BTreeSet<String> = unobservable_all.iter().step_by(2).cloned().collect();

    assert_eq!(
        observable_raw.len() + unobservable_all.len() / 2,
        ROWS.len(),
        "roster id COUNT must equal ROWS.len() ({}) -- a duplicate id, or a row \
         hidden inside a /* */ comment, is invisible to the set comparison below. \
         observable raw count: {}, unobservable raw pair count: {}",
        ROWS.len(),
        observable_raw.len(),
        unobservable_all.len() / 2
    );

    let rows_ids: BTreeSet<String> = ROWS.iter().map(|r| r.id.to_string()).collect();
    let served_ids: BTreeSet<String> = ROWS
        .iter()
        .filter(|r| matches!(r.class, DecisionClass::Served { .. }))
        .map(|r| r.id.to_string())
        .collect();

    let union: BTreeSet<String> = observable.union(&unobservable).cloned().collect();

    let missing_from_roster: Vec<&String> = rows_ids.difference(&union).collect();
    let extra_in_roster: Vec<&String> = union.difference(&rows_ids).collect();
    assert!(
        missing_from_roster.is_empty() && extra_in_roster.is_empty(),
        "decision_coverage.rs's OBSERVABLE_ROW_IDS ∪ UNOBSERVABLE_ROW_IDS must equal \
         decision_site_walk.rs's ROWS ids exactly. In ROWS but missing from the \
         roster: {missing_from_roster:?}. In the roster but not a ROWS id: \
         {extra_in_roster:?}"
    );

    let observable_not_served: Vec<&String> = observable.difference(&served_ids).collect();
    let served_not_observable: Vec<&String> = served_ids.difference(&observable).collect();
    assert!(
        observable_not_served.is_empty() && served_not_observable.is_empty(),
        "OBSERVABLE_ROW_IDS must equal EXACTLY the ROWS ids whose class is Served. In \
         OBSERVABLE_ROW_IDS but not Served in ROWS: {observable_not_served:?}. Served \
         in ROWS but missing from OBSERVABLE_ROW_IDS: {served_not_observable:?}"
    );

    assert!(
        union.len() >= MIN_ROWS,
        "the combined roster shrank to {} (< MIN_ROWS {MIN_ROWS}) -- rows may be \
         added, never removed",
        union.len()
    );
    assert!(
        observable.len() >= 5,
        "OBSERVABLE_ROW_IDS must have at least the 5 known Served rows, got {}",
        observable.len()
    );
    assert!(
        !unobservable.is_empty(),
        "UNOBSERVABLE_ROW_IDS must not be empty"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// The total classification of `pub enum Effect` (`OOS-DX28-1`)
// ─────────────────────────────────────────────────────────────────────────────

/// Declared `Effect` variants that are deliberately NOT decision-site rows, each with the
/// reason. See [`every_effect_variant_is_classified`] for what this list is for and why a
/// bare `MIN_ROWS` ratchet was not enough.
///
/// Two kinds of entry live here and each says which it is:
///
/// * **no decision** — every question the variant's resolution has to answer is answered by
///   its own payload, by a CR 115 target announced at cast/activation time, by PB-DX28's
///   `EffectTarget::ChosenObject` channel, or by a nested `Effect` that this same RECURSIVE
///   walk still reaches (the combinators);
/// * **gated elsewhere** — the variant does take a choice, and a named gate holds its
///   deck-legal population at zero. `ROWS` already carries `DecisionClass::Gated` for two
///   such variants; these are the ones with the gate and no row.
const NON_ROW_EFFECT_VARIANTS: &[(&str, &str)] = &[
    (
        "AddCounter",
        "`counter`, `count` and an `EffectTarget` are all in the payload.",
    ),
    (
        "AddCounterAmount",
        "`counter`, `count` and an `EffectTarget` are all in the payload.",
    ),
    (
        "AddMana",
        "The exact `ManaCost` produced is in the payload — no colour is chosen.",
    ),
    (
        "AddManaAnyColor",
        "GATED by SR-33's `core::effect_choose_gate::no_complete_def_uses_an_any_color_mana_stub`, unless the def registers a real served `any_color` tap mana ability (that gate's own documented condition, with its documented hole).",
    ),
    (
        "AddManaAnyColorRestricted",
        "GATED by SR-33's `no_complete_def_uses_an_any_color_mana_stub`, which flags this variant UNCONDITIONALLY on any `Complete` def.",
    ),
    (
        "AddManaChoice",
        "GATED, not undecided. `Effect::AddManaChoice` adds one COLORLESS mana and ignores its `count`; SR-33's `core::effect_choose_gate::no_complete_def_uses_the_add_mana_choice_stub` bars it from every `Complete` def, so its deck-legal population is held at zero by a gate rather than by authoring discipline. That is the treatment ROWS gives `Choose` and `MayPayOrElse` — minus the row.",
    ),
    (
        "AddManaOfAnyColorAmount",
        "GATED by SR-33's `no_complete_def_uses_an_any_color_mana_stub`, which flags this variant UNCONDITIONALLY on any `Complete` def.",
    ),
    (
        "AddManaOfChosenColor",
        "CONSUMES a decision rather than making one: it reads `obj.chosen_color`, recorded earlier by a `ChooseColor` (CR 614.12a). That choice is the `choose_color_or_type` ROW.",
    ),
    (
        "AddManaRestricted",
        "`mana` and `restriction` are both in the payload (CR 106.6b).",
    ),
    (
        "AddManaScaled",
        "`color` and `count` are both in the payload; no colour is chosen at resolution.",
    ),
    (
        "AdditionalCombatPhase",
        "CR 506.1 — an extra phase is inserted; `followed_by_main` is in the payload.",
    ),
    (
        "AdditionalLandPlay",
        "CR 305.2 — raises the land-play allowance; nothing is chosen.",
    ),
    (
        "ApplyContinuousEffect",
        "Installs the payload's `ContinuousEffectDef` verbatim.",
    ),
    (
        "AttachEquipment",
        "Both the equipment and the creature are `EffectTarget`s (CR 702.6a's target is announced at activation).",
    ),
    (
        "AttachFortification",
        "Both the fortification and the land are `EffectTarget`s (CR 702.67a).",
    ),
    (
        "BecomeCopyOf",
        "Both the copier and the copied object are `EffectTarget`s; `duration` is in the payload. CR 707.10's choose-new-targets clause applies to a copy of a SPELL, which is `CopySpellOnStack`, on the candidate list below.",
    ),
    (
        "BecomeMonarch",
        "CR 720.2 — the new monarch is the payload's `PlayerTarget`; nothing is picked.",
    ),
    (
        "Bite",
        "Both creatures are `EffectTarget`s (CR 701.12a's fight shape, one-directional).",
    ),
    (
        "BounceAll",
        "A mass effect over a `TargetFilter` (optionally bounded by `max_toughness_amount`); every match is returned.",
    ),
    (
        "Cloak",
        "CR 702.176a cloaks the TOP card of the library; `player` is in the payload.",
    ),
    (
        "CoinFlip",
        "CR 705 — a random outcome is not a player choice, and both branch `Effect`s are reached by the recursive walk.",
    ),
    (
        "Conditional",
        "COMBINATOR — it carries no decision of its own. Its `if_true`/`if_false` branches are nested `Effect`s, and `json_contains_variant` is a RECURSIVE walk, so a decision inside a branch is still found and attributed to that branch's own row.",
    ),
    (
        "CounterSpell",
        "The spell or ability is an `EffectTarget`; `exile_instead` is in the payload.",
    ),
    (
        "CreateEmblem",
        "CR 114.1 — the emblem's abilities are the payload.",
    ),
    (
        "CreateToken",
        "The token's whole characteristic set is the payload's `TokenSpec`; nothing about it is chosen at resolution.",
    ),
    (
        "CreateTokenAndAttachSource",
        "The token is the payload's `TokenSpec` and the host is the source.",
    ),
    (
        "CreateTokenCopy",
        "The copied permanent is an `EffectTarget`; every deviation from a plain copy (`gains_haste`, `except_not_legendary`, …) is a payload flag.",
    ),
    (
        "DealDamage",
        "Payload-determined: `amount`, `source` and an `EffectTarget` recipient. Which object or player takes the damage is a CR 115 target (or PB-DX28's `ChosenObject` channel), not a resolution-time pick.",
    ),
    (
        "DestroyAll",
        "A mass effect over a `TargetFilter`: EVERY match is destroyed, so there is no 'which'. (The CR 404.3 order simultaneous deaths enter a graveyard IS a choice; it is the separate class-B site the `wheel_hand` row's own note files as `OOS-DP10-10`, and it is not a property of this variant.)",
    ),
    (
        "DestroyAndReanimate",
        "Both halves name `EffectTarget`s; the destination is fixed by the variant.",
    ),
    (
        "DestroyPermanent",
        "The permanent is an `EffectTarget` — a CR 115 target announced at cast/activation, or PB-DX28's `EffectTarget::ChosenObject` channel, both separately served.",
    ),
    (
        "DetachEquipment",
        "The equipment is an `EffectTarget`; CR 301.5c unattaching asks nothing.",
    ),
    (
        "DrainLife",
        "Payload-determined: `amount`; the two halves are the source's controller and the effect's target.",
    ),
    (
        "DrawCards",
        "CR 121.1 draws from the TOP of the library; `count` and `player` are in the payload, so there is no 'which card'.",
    ),
    (
        "ExchangeControl",
        "Both permanents are `EffectTarget`s; `duration` is in the payload (CR 613.1c).",
    ),
    (
        "ExileAll",
        "A mass effect over a `TargetFilter`; every match is exiled.",
    ),
    (
        "ExileObject",
        "The object is an `EffectTarget` — a CR 115 target, or PB-DX28's `ChosenObject` channel.",
    ),
    (
        "ExileSourceAndReturnTransformed",
        "Acts on the SOURCE alone; no participant is chosen (CR 701.28a).",
    ),
    (
        "ExileWithDelayedReturn",
        "The permanent is an `EffectTarget`; `return_timing`, `return_tapped` and `return_to` are all in the payload.",
    ),
    (
        "ExtraTurn",
        "CR 500.7 — `player` and `count` are in the payload. (WHEN the extra turn is taken is fixed; the APNAP question `extra_turns` raises is a turn-structure property, not a resolution-time pick.)",
    ),
    (
        "Fight",
        "Both creatures are `EffectTarget`s (CR 701.12a).",
    ),
    (
        "Flicker",
        "The permanent is an `EffectTarget`; CR 400.7 makes the returning object a new one and the return is automatic.",
    ),
    (
        "ForEach",
        "COMBINATOR — see `Conditional`. (The CR 101.4 APNAP order of a per-player `ForEach` is `OOS-DP9-8`'s subject, closed by PB-DX15a, and is a property of the walk rather than of this variant.)",
    ),
    (
        "GainControl",
        "The permanent is an `EffectTarget`; `duration` is in the payload.",
    ),
    (
        "GainLife",
        "Payload-determined: `amount` and a `PlayerTarget`.",
    ),
    (
        "Goad",
        "The creature is an `EffectTarget`; CR 701.38a imposes a requirement on a later declaration rather than asking anything now.",
    ),
    (
        "GrantFlash",
        "`filter` and `duration` are in the payload; CR 702.8a is granted, not chosen.",
    ),
    (
        "GrantPlayerProtection",
        "`player`, `qualities` and `duration` are all in the payload.",
    ),
    (
        "Investigate",
        "CR 701.32a names the Clue token exactly; only `count` varies and it is in the payload.",
    ),
    (
        "LivingDeath",
        "CR 701.21/400 — every creature card in every graveyard and every creature on the battlefield is exchanged; the set is total, so there is no 'which'.",
    ),
    (
        "LoseLife",
        "Payload-determined: `amount` and a `PlayerTarget`.",
    ),
    (
        "Manifest",
        "CR 701.34a manifests the TOP card of the library; `player` is in the payload.",
    ),
    (
        "Meld",
        "CR 701.37 — the partner is fixed by the source's `melded_card_id`, so there is no pick.",
    ),
    (
        "MillCards",
        "CR 701.13a mills from the TOP; `count` and `player` are in the payload.",
    ),
    (
        "MoveZone",
        "The object is an `EffectTarget` (CR 115 target, or PB-DX28's `ChosenObject`) and the destination is the payload's `ZoneTarget`.",
    ),
    (
        "Nothing",
        "The no-op. There is nothing to decide, which is the point of the variant.",
    ),
    (
        "PreventAllCombatDamage",
        "A blanket prevention shield; no participant is chosen.",
    ),
    (
        "PreventCombatDamageFromOrTo",
        "The permanent is an `EffectTarget`; the two direction flags are in the payload.",
    ),
    (
        "PreventNextUntap",
        "The permanent is an `EffectTarget`; the shield is created, not chosen (CR 614.1).",
    ),
    (
        "Regenerate",
        "The permanent is an `EffectTarget`; CR 701.15a's shield is created, not chosen.",
    ),
    (
        "RegisterReplacementEffect",
        "Installs the payload's replacement `modification` under the payload's `trigger`.",
    ),
    (
        "RemoveCounter",
        "`counter`, `count` and an `EffectTarget` are all in the payload.",
    ),
    (
        "RemoveFromCombat",
        "The creature is an `EffectTarget`; CR 506.4 removal asks nothing.",
    ),
    (
        "Repeat",
        "COMBINATOR — see `Conditional`; the nested `effect` is reached by the same recursive walk.",
    ),
    (
        "RollDice",
        "CR 706 — as `CoinFlip`. Its `results` payload is a `Vec<(u32, u32, Effect)>`; the walk runs over SERIALIZED JSON, so the nested effects inside those tuples are still visited (PB-DX26's `RollDice` nesting lesson, which a `Box`/`Vec` field count cannot see).",
    ),
    (
        "Sequence",
        "COMBINATOR — a tuple variant holding `Vec<Effect>`; every member is reached by the recursive walk.",
    ),
    (
        "SetNoMaximumHandSize",
        "CR 402.2 — `player` is in the payload; the cleanup discard it removes is `ENG-1`'s row.",
    ),
    (
        "SetReturnToHandAtEndStep",
        "A delayed trigger on the source itself; no participant is chosen.",
    ),
    (
        "Shuffle",
        "CR 701.20 — randomisation is not a player choice.",
    ),
    (
        "SolveCase",
        "CR 715 — solving is decided by the case's own condition, not by a player.",
    ),
    (
        "Suspect",
        "The creature is an `EffectTarget`; CR 701.59a confers two static abilities.",
    ),
    (
        "TakeTheInitiative",
        "CR 725.2 — the controller takes the initiative. (It then ventures, and THAT half is `VentureIntoDungeon`'s row on the candidate list below; this variant contributes no choice of its own.)",
    ),
    (
        "TapPermanent",
        "The permanent is an `EffectTarget` — a CR 115 target, or PB-DX28's `ChosenObject`.",
    ),
    (
        "TransformSelf",
        "CR 701.28a — the source transforms; no participant is chosen.",
    ),
    (
        "Unsuspect",
        "The creature is an `EffectTarget`; CR 701.59b removes the two static abilities.",
    ),
    (
        "UntapAll",
        "A mass effect over a `TargetFilter`; every match untaps.",
    ),
    (
        "UntapPermanent",
        "The permanent is an `EffectTarget` — a CR 115 target, or PB-DX28's `ChosenObject`.",
    ),
    (
        "WinGame",
        "CR 104.2a — the game ends; no participant and no ordering is chosen.",
    ),
];

/// Declared `Effect` variants whose engine implementation VISIBLY takes a choice the CR gives
/// a player, and which have no `ROWS` row.
///
/// These are filed rather than claimed. Each is stated with the CR rule and the line of
/// `crates/engine/src` that takes the choice, and none of them is asserted to be harmless —
/// the honest position is *"a row is probably owed here and the audit has not adjudicated
/// one"*, which is a different thing from `NON_ROW_EFFECT_VARIANTS`' *"no row is owed"*.
///
/// They are separated from `NON_ROW_EFFECT_VARIANTS` so that a future batch promoting one to
/// a `ROWS` row moves it from this list to the table, and the partition assertion below keeps
/// working either way. Note what this list does NOT do: it does not change any count, any
/// `BASELINE`, or any behaviour. It converts an undocumented floor into a stated bound.
const UNADJUDICATED_DECISION_CANDIDATES: &[(&str, &str)] = &[
    (
        "AddManaMatchingType",
        "CR 106.12a *\"add one mana of any type that land produced\"* is the PLAYER's choice; `effects/mod.rs` takes `ctx.mana_produced.first()` and falls back to Colorless. Unlike its four `AddMana*` siblings above it is NOT covered by SR-33's `no_complete_def_uses_an_any_color_mana_stub` (checked: that gate names `AddManaChoice`, `AddManaAnyColor`, `AddManaAnyColorRestricted` and `AddManaOfAnyColorAmount`, and not this one), so nothing holds its deck-legal population at zero either.",
    ),
    (
        "CopySpellOnStack",
        "CR 707.10 — *\"the controller of the copy may choose new targets for it.\"* `effects/mod.rs`'s own comment at the copy site says *\"choose-new-targets deferred to M10\"*, and `card_definition.rs`'s `CopySpellOnStack` doc says CR 707.10c is *\"deterministic — copies keep original targets\"*. Cross-filed with `OOS-DX54-2`.",
    ),
    (
        "PlayExiledCard",
        "CR 702.76a (hideaway) prints *\"you may play that card without paying its mana cost\"*; the engine plays it unconditionally. The optionality is the decision, which is the `may_pay_then_effect` row's family reached through a different variant.",
    ),
    (
        "PutLandFromHandOntoBattlefield",
        "Self-declared in source: *\"Deterministic land selection: pick the land card with the lowest ObjectId. In a real game with human players, this would require a choice command.\"* CR 701.17 — the player chooses which land.",
    ),
    (
        "ReturnAllFromGraveyardToBattlefield",
        "Only under its `unique_names: true` arm (Eerie Ultimatum, CR 701.17): the printed card lets the player choose ANY NUMBER of permanent cards with different names, and `effects/mod.rs` *\"keep[s] only the lowest ObjectId per name\"*. With `unique_names: false` the set is total and there is nothing to choose, so this entry is a decision site on one arm and not on the other — which is why it is a candidate rather than a row, and why the row that eventually covers it must be predicate-qualified rather than a bare variant match.",
    ),
    (
        "VentureIntoDungeon",
        "TWO engine-taken choices, both visible in `rules::engine::handle_venture_into_dungeon`. CR 701.49a gives the player the choice of WHICH dungeon; the engine hard-codes `DungeonId::LostMineOfPhandelver`. CR 701.49b gives the player the choice of next room when a room has more than one exit; the engine takes `exits.first()`.",
    ),
];

/// `ROWS` ids whose predicate names no `Effect` variant at all, with the reason.
///
/// The two compound rows: both are `AbilityDefinition`-shaped, qualified by a FIELD of the
/// matched node (`targets` non-empty / `modes` non-null) rather than by a variant name, which
/// is why [`find_variant_nodes`] exists. A row that fell out of this list AND matched no
/// declared `Effect` variant would be a row whose predicate has gone dead — most likely
/// because the variant it names was renamed — and the check below says so.
const ROWS_WITH_NO_EFFECT_VARIANT: &[(&str, &str)] = &[
    (
        "triggered_targets",
        "CR 603.3d — an `AbilityDefinition::Triggered` node with a non-empty `targets` field. \
         Not a variant-name match; see this module's doc.",
    ),
    (
        "modal_trigger",
        "CR 603.3c — an `AbilityDefinition::Triggered` node with a non-null `modes` field.",
    ),
];

/// The `Effect` variants each `ROWS` row's predicate actually responds to, MEASURED by
/// executing the predicate rather than read off a hand-written list.
///
/// For every declared variant name, this builds the one-key object `{"<Variant>": {}}` —
/// the serde external tagging shape [`json_contains_variant`] matches — and asks each row's
/// predicate. So the "covered" side of the classification below is derived by execution and
/// cannot drift from what the predicates do: a predicate rewritten to name a different
/// variant moves this map on the next run, with no list to remember to update.
fn effect_variants_named_by_rows() -> BTreeMap<String, BTreeSet<&'static str>> {
    let declared = crate::pb_dx57_declared_source::declared_enum_variants(
        crate::pb_dx57_declared_source::CARD_DEFINITION_RS,
        "Effect",
    );
    let mut out: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    for name in declared {
        let mut obj = serde_json::Map::new();
        obj.insert(name.clone(), Value::Object(serde_json::Map::new()));
        let probe = Value::Object(obj);
        let hits: BTreeSet<&'static str> = ROWS
            .iter()
            .filter(|r| (r.predicate)(&probe))
            .map(|r| r.id)
            .collect();
        if !hits.is_empty() {
            out.insert(name, hits);
        }
    }
    out
}

#[test]
/// `OOS-DX28-1` — **every declared `Effect` variant is either a decision-site row, an
/// explicitly non-row variant with a reason, or a filed candidate.**
///
/// ## What was wrong before
///
/// `ROWS` is a hand-maintained table of 22 entries, and the only thing guarding it was
/// `decision_gate.rs`'s `const MIN_ROWS: usize = 22`, asserted with `>=` — *"rows may be
/// added, never removed"*. **Nothing compared the row set to `pub enum Effect`'s
/// declaration.** A new decision-carrying `Effect` variant therefore escaped the whole
/// decision-point audit in silence, and the ratchet stayed green, because a ratchet's slack
/// IS its blind spot: `MIN_ROWS` measures the table's SIZE and says nothing about the
/// population the table is supposed to cover. The stage-0 census calls this the
/// widest-blast-radius member of the `OOS-DX28-1` class, and it is the same failure as
/// `TARGET_FILTER_FIELDS`: a hand-maintained fingerprint that goes blind on declaration
/// growth with no compile error.
///
/// ## Why not set equality
///
/// Set equality against `Effect` would be wrong: most of the 106 variants carry no player
/// decision, so `ROWS == declared` is not the property anyone wants. The correct pin is a
/// **TOTAL CLASSIFICATION** — `core::pb_dx50_copy_additional_cost_roster::r1`'s shape, which
/// the census calls the cleanest instance of this repair in the tree. Every declared variant
/// must be in exactly one of three places:
///
/// 1. named by some `ROWS` predicate (derived by EXECUTION — see
///    [`effect_variants_named_by_rows`], not by a second hand-written list);
/// 2. [`NON_ROW_EFFECT_VARIANTS`], with the reason no row is owed;
/// 3. [`UNADJUDICATED_DECISION_CANDIDATES`], with the CR rule and the engine line that takes
///    the choice.
///
/// A 107th variant is then a red test whose message names the choice its author has to make.
///
/// ## Stated bounds
///
/// * **A row can name something that is not an `Effect` variant, and two do.**
///   `ROWS_WITH_NO_EFFECT_VARIANT` holds the two compound `AbilityDefinition`-shaped rows;
///   `choose_color_or_type` additionally matches `ReplacementModification::ChooseColor`,
///   which is outside this pin's declaration and is not counted here. This row pins the
///   `Effect` axis only, and says so rather than implying it covers the audit's whole domain.
/// * **Membership of `UNADJUDICATED_DECISION_CANDIDATES` is a CLAIM about the engine, and
///   each entry names the file and the sentence it rests on.** Two of the six quote an
///   in-source comment that says the choice is deferred; a reader who disagrees with an
///   entry should move it, not delete the list.
/// * This row asserts nothing about `decision_gate.rs`'s `BASELINE` and moves no count.
/// * **"In silence" is precise, not rhetorical.** Adding an `Effect` variant does redden the
///   two WIRE gates (`hash_schema::declaration_fingerprint_is_pinned`,
///   `protocol_schema::protocol_schema_fingerprint_is_pinned`) -- measured, by planting one.
///   But those say *"the wire moved"*, which is answered by bumping a version number, and
///   they say nothing about whether the decision-point audit still covers the enum. Before
///   this row, the audit's own coverage was the thing that could move without any test
///   mentioning it.
fn every_effect_variant_is_classified() {
    // **THE TWO LISTS CARRY OPPOSITE CLAIMS AND A PARTITION CANNOT TELL THEM APART.** The
    // adversarial pass moved `VentureIntoDungeon` from `UNADJUDICATED_DECISION_CANDIDATES` into
    // `NON_ROW_EFFECT_VARIANTS` with a fabricated reason and **the whole `core` target stayed
    // green** — the union still equals the declaration, the lists are still disjoint, and the
    // reason is still non-empty. What changed is the CLAIM: *"the engine takes a CR choice here
    // and no audit row covers it"* became *"there is no player choice to hook"*, which is the
    // difference between a filed gap and a closed question, laundered by an edit no gate saw.
    //
    // So the candidate set is pinned BY NAME. Moving a variant OUT of it is a claim that a
    // decision-point gap has been resolved, and that is a re-adjudication with a reason, not a
    // bookkeeping edit. Adding one is free (a new gap should be easy to file) and the pin says
    // so — this is a subset assertion in the permissive direction and an equality in the
    // dangerous one.
    const PINNED_CANDIDATES: &[&str] = &[
        "AddManaMatchingType",
        "CopySpellOnStack",
        "PlayExiledCard",
        "PutLandFromHandOntoBattlefield",
        "ReturnAllFromGraveyardToBattlefield",
        "VentureIntoDungeon",
    ];
    let live_candidates: std::collections::BTreeSet<&str> = UNADJUDICATED_DECISION_CANDIDATES
        .iter()
        .map(|(v, _)| *v)
        .collect();
    let pinned_candidates: std::collections::BTreeSet<&str> =
        PINNED_CANDIDATES.iter().copied().collect();
    let removed: Vec<&&str> = pinned_candidates.difference(&live_candidates).collect();
    assert!(
        removed.is_empty(),
        "decision-point candidate(s) {removed:?} have LEFT \
         UNADJUDICATED_DECISION_CANDIDATES. That is not bookkeeping: membership is the claim \
         *the engine takes a CR choice here and no audit row covers it*, and removing a member \
         asserts the gap is resolved. If it really is — a ROWS row was added, or the CR turns \
         out to grant no choice — say which in the same commit and update this pin. If it was \
         merely MOVED to NON_ROW_EFFECT_VARIANTS, the claim has been INVERTED, and the \
         partition below is invariant under exactly that move (measured: the whole core target \
         stayed green)."
    );

    let declared = crate::pb_dx57_declared_source::declared_enum_variants(
        crate::pb_dx57_declared_source::CARD_DEFINITION_RS,
        "Effect",
    );
    assert!(
        declared.len() >= 90,
        "the `pub enum Effect` parse returned only {} variants — the parser is broken and \
         every assertion below is vacuous",
        declared.len()
    );

    let covered_map = effect_variants_named_by_rows();
    let covered: BTreeSet<String> = covered_map.keys().cloned().collect();
    assert!(
        covered.len() >= 20,
        "only {} declared `Effect` variant(s) are named by any ROWS predicate. That is not a \
         table that shrank — every predicate is a `json_contains_variant` text match, so this \
         means the probe shape stopped matching and the derivation has gone vacuous.",
        covered.len()
    );

    let non_row: BTreeSet<String> = NON_ROW_EFFECT_VARIANTS
        .iter()
        .map(|(n, _)| (*n).to_string())
        .collect();
    let candidates: BTreeSet<String> = UNADJUDICATED_DECISION_CANDIDATES
        .iter()
        .map(|(n, _)| (*n).to_string())
        .collect();

    assert_eq!(
        non_row.len(),
        NON_ROW_EFFECT_VARIANTS.len(),
        "NON_ROW_EFFECT_VARIANTS names the same variant twice"
    );
    assert_eq!(
        candidates.len(),
        UNADJUDICATED_DECISION_CANDIDATES.len(),
        "UNADJUDICATED_DECISION_CANDIDATES names the same variant twice"
    );

    // Pairwise disjoint — a variant cannot be both covered and excused, and a candidate that
    // has been promoted to a row must be REMOVED from the candidate list rather than left in
    // both places, or the list stops meaning "not yet adjudicated".
    for (a, an, b, bn) in [
        (
            &covered,
            "a ROWS predicate",
            &non_row,
            "NON_ROW_EFFECT_VARIANTS",
        ),
        (
            &covered,
            "a ROWS predicate",
            &candidates,
            "UNADJUDICATED_DECISION_CANDIDATES",
        ),
        (
            &non_row,
            "NON_ROW_EFFECT_VARIANTS",
            &candidates,
            "UNADJUDICATED_DECISION_CANDIDATES",
        ),
    ] {
        let both: Vec<&String> = a.intersection(b).collect();
        assert!(
            both.is_empty(),
            "`Effect` variant(s) classified BOTH by {an} and by {bn}: {both:?}"
        );
    }

    let classified: BTreeSet<String> = covered
        .union(&non_row)
        .cloned()
        .collect::<BTreeSet<String>>()
        .union(&candidates)
        .cloned()
        .collect();

    assert_eq!(
        classified,
        declared,
        "`OOS-DX28-1`: `pub enum Effect` and the decision-site classification have diverged.\n\
         \n  UNCLASSIFIED (declared, but named by no ROWS predicate and on neither list): \
         {:?}\n    → decide, in this order: does the CR give a player a choice at this \
         effect's resolution? If YES and the engine makes it, add a `ROWS` row (and its \
         predicate) or, if that is a bigger job than this batch, add it to \
         UNADJUDICATED_DECISION_CANDIDATES with the CR rule and the engine line. If NO, add \
         it to NON_ROW_EFFECT_VARIANTS with the reason. Do NOT delete this assertion: the \
         whole point is that a new decision-carrying variant used to escape the audit in \
         silence while `MIN_ROWS` stayed green.\n\
         \n  DEAD (classified here, absent from the declaration): {:?}\n    → a rename or a \
         removal. A renamed variant is the dangerous half: every `ROWS` predicate is a TEXT \
         match on a variant name, so a rename makes that row's predicate match nothing, its \
         population silently fall to zero, and `decision_gate.rs`'s `BASELINE` shrink without \
         anything saying why.",
        declared.difference(&classified).collect::<Vec<_>>(),
        classified.difference(&declared).collect::<Vec<_>>()
    );

    // Every row's predicate must still respond to something, or be one of the two compound
    // rows. This is the RENAME half stated directly: a row whose predicate matches no
    // declared variant is a dead row, and it fails here rather than as an unexplained
    // BASELINE shrink.
    let rows_with_no_variant: BTreeSet<&str> = ROWS_WITH_NO_EFFECT_VARIANT
        .iter()
        .map(|(id, _)| *id)
        .collect();
    let live_row_ids: BTreeSet<&str> = covered_map.values().flatten().copied().collect();
    let dead: Vec<&str> = ROWS
        .iter()
        .map(|r| r.id)
        .filter(|id| !live_row_ids.contains(id) && !rows_with_no_variant.contains(id))
        .collect();
    assert!(
        dead.is_empty(),
        "ROWS row(s) whose predicate names no declared `Effect` variant: {dead:?}. Either \
         the variant was renamed (fix the predicate) or the row is genuinely not \
         variant-shaped (add it to ROWS_WITH_NO_EFFECT_VARIANT with the reason, as the two \
         compound rows are)."
    );
    // ...and the converse, so that list cannot rot into a bucket for rows that have since
    // become variant-shaped.
    let stale: Vec<&str> = rows_with_no_variant
        .iter()
        .copied()
        .filter(|id| live_row_ids.contains(id) || !ROWS.iter().any(|r| r.id == *id))
        .collect();
    assert!(
        stale.is_empty(),
        "ROWS_WITH_NO_EFFECT_VARIANT entr(ies) that are no longer true — either the row now \
         DOES name a declared `Effect` variant, or the row id no longer exists in ROWS: \
         {stale:?}"
    );

    // Every reason is a real reason. An allowlist whose justification is not checked is a
    // comment, which is how `OOS-DX28-1`'s own class survives.
    for (name, why) in NON_ROW_EFFECT_VARIANTS
        .iter()
        .chain(UNADJUDICATED_DECISION_CANDIDATES)
        .chain(ROWS_WITH_NO_EFFECT_VARIANT)
    {
        assert!(
            why.len() > 40,
            "the classification entry for `{name}` carries no real reason: {why:?}"
        );
    }
}
