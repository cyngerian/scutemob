//! PB-DX28 §1 (`OOS-DX4-6`): the untargeted-choice channel's roster gates.
//!
//! Four checks, per the batch's own plan (`pb-plan-DX28-part2.md` §3):
//!
//! * **R1** — the exact set of corpus defs naming `EffectTarget::ChosenObject`,
//!   pinned by NAME. A pin, so an 18th use is a deliberate act.
//! * **R2** — every `ChosenObject` filter in the corpus sets only axes
//!   `filter_matches_object_untargeted` implements (fails CLOSED on
//!   `max_cmc_amount`/`min_cmc_amount`). Non-vacuity floor: the set is
//!   non-empty and at least one member sets `controller`.
//! * **R3** — no migrated def declares a `TargetRequirement` for the ability
//!   that now uses `ChosenObject` (the migration is complete, not additive) —
//!   with `rewind` as the stated exception (its slot 0 `TargetSpell` is a
//!   REAL printed target and stays).
//! * **R4** — inverse axis: no `Complete` def still pairs a declared
//!   `TargetRequirement` slot count strictly greater than its oracle text's
//!   `"target"` word count, outside a named allowlist of the REFUTED rows in
//!   `pb-plan-DX28.md` §0.1. This is the census, frozen, so the class cannot
//!   silently regrow.
//!
//! Reuses `decision_site_walk.rs`'s canonical JSON walk (`find_variant_nodes`,
//! `def_contains_variant`, `PROSE_FIELDS`, `is_effectively_complete`) rather
//! than a second hand-written tree walk — PB-DP10's own lesson: a hand-written
//! walk is a reachability claim and needs the same enumeration a match arm
//! does; the serialized-JSON walk reaches every field by construction.

use crate::decision_site_walk::{
    def_contains_variant, find_variant_nodes, is_effectively_complete, PROSE_FIELDS,
};
use mtg_engine::all_cards;
use serde_json::Value;
use std::collections::BTreeSet;

// ── R1: the pinned roster ────────────────────────────────────────────────────

/// The exact 17 corpus defs naming `EffectTarget::ChosenObject`, by name.
const CHOSEN_OBJECT_MEMBERS: &[&str] = &[
    "Azorius Chancery",
    "Boros Garrison",
    "Cloud of Faeries",
    "Dimir Aqueduct",
    "Frantic Search",
    "Golgari Rot Farm",
    "Gruul Turf",
    "Izzet Boilerworks",
    "Orzhov Basilica",
    "Rakdos Carnarium",
    "Rewind",
    "Selesnya Sanctuary",
    "Shrieking Drake",
    "Simic Growth Chamber",
    "Sword of Truth and Justice",
    "Takenuma, Abandoned Mire",
    "Whitemane Lion",
];

#[test]
fn r1_chosen_object_roster_is_pinned() {
    let live: BTreeSet<String> = all_cards()
        .into_iter()
        .filter(|d| def_contains_variant(d, "ChosenObject"))
        .map(|d| d.name.clone())
        .collect();
    let pinned: BTreeSet<String> = CHOSEN_OBJECT_MEMBERS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        live,
        pinned,
        "EffectTarget::ChosenObject roster moved -- an 18th use (or a dropped one) must be \
         a deliberate act, confirmed here. In pinned but not live: {:?}. In live but not \
         pinned: {:?}",
        pinned.difference(&live).collect::<Vec<_>>(),
        live.difference(&pinned).collect::<Vec<_>>(),
    );
    assert_eq!(
        CHOSEN_OBJECT_MEMBERS.len(),
        17,
        "the roster itself must not silently shrink"
    );
}

// ── R2: every ChosenObject filter sets only supported axes ──────────────────

#[test]
fn r2_chosen_object_filters_set_only_supported_axes() {
    let mut filters: Vec<Value> = Vec::new();
    for def in all_cards() {
        if !def_contains_variant(&def, "ChosenObject") {
            continue;
        }
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        for node in find_variant_nodes(&json, "ChosenObject") {
            let filter = node
                .get("filter")
                .unwrap_or_else(|| panic!("ChosenObject node in {} has no filter", def.name));
            filters.push(filter.clone());
        }
    }
    assert!(
        !filters.is_empty(),
        "non-vacuity: the corpus must carry at least one ChosenObject filter"
    );
    // Fail-closed axes: filter_matches_object_untargeted does NOT implement
    // max_cmc_amount / min_cmc_amount (EffectContext-resolved). Any corpus
    // filter setting either would be silently mishandled (rejected outright,
    // not narrowed) -- catch it here, at author time, not in production.
    for f in &filters {
        for unsupported in ["max_cmc_amount", "min_cmc_amount"] {
            let is_set = f.get(unsupported).is_some_and(|v| !v.is_null());
            assert!(
                !is_set,
                "a ChosenObject filter sets {unsupported:?}, which \
                 filter_matches_object_untargeted does not support (EffectContext-resolved) \
                 -- it will be rejected outright, not narrowed: {f:?}"
            );
        }
    }
    // Non-vacuity of the positive claim: at least one member actually narrows
    // by controller (the ten Karoos + shrieking_drake/whitemane_lion/sword_of_
    // truth_and_justice all do -- "a land/creature YOU control").
    let any_sets_controller = filters
        .iter()
        .any(|f| f.get("controller").is_some_and(|c| c != "Any"));
    assert!(
        any_sets_controller,
        "non-vacuity: at least one ChosenObject filter must narrow by controller"
    );
}

// ── R3: the migration is complete, not additive ──────────────────────────────

/// Every `AbilityDefinition::Triggered` / `Spell` / `Activated` node's OWN
/// `targets` list, as parsed JSON arrays, paired with whether that SAME node
/// also contains a `ChosenObject`. Returns `(has_chosen_object, targets_len)`
/// for every Triggered/Spell/Activated node in the def. `Activated` is
/// searched too -- `takenuma_abandoned_mire`'s Channel ability is an
/// `AbilityDefinition::Activated`, not a Triggered or Spell.
fn ability_target_shapes(def: &mtg_engine::CardDefinition) -> Vec<(bool, usize)> {
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    let mut out = Vec::new();
    for variant in ["Triggered", "Spell", "Activated"] {
        for node in find_variant_nodes(&json, variant) {
            let targets_len = node
                .get("targets")
                .and_then(|t| t.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let has_chosen_object = node
                .get("effect")
                .is_some_and(|e| json_contains_key(e, "ChosenObject"));
            out.push((has_chosen_object, targets_len));
        }
    }
    out
}

fn json_contains_key(v: &Value, key: &str) -> bool {
    match v {
        Value::Object(map) => map
            .iter()
            .any(|(k, child)| k == key || json_contains_key(child, key)),
        Value::Array(items) => items.iter().any(|i| json_contains_key(i, key)),
        _ => false,
    }
}

#[test]
fn r3_migration_is_complete_not_additive() {
    for name in CHOSEN_OBJECT_MEMBERS {
        let def = all_cards()
            .into_iter()
            .find(|d| &d.name == name)
            .unwrap_or_else(|| panic!("{name} should be in the corpus"));
        let shapes = ability_target_shapes(&def);
        let chosen_object_nodes: Vec<_> = shapes.iter().filter(|(has, _)| *has).collect();
        assert!(
            !chosen_object_nodes.is_empty(),
            "{name}: R1 found a ChosenObject but no Triggered/Spell node carries it -- is it \
             on an AddCounter/UntapPermanent inside a Sequence the walk didn't reach?"
        );
        if *name == "Rewind" {
            // The stated exception: slot 0 `TargetSpell` is a REAL printed target
            // ("Counter target spell.") and stays. Exactly one requirement.
            for (has_chosen_object, targets_len) in &shapes {
                if *has_chosen_object {
                    assert_eq!(
                        *targets_len, 1,
                        "Rewind's Spell node must carry exactly one TargetRequirement \
                         (slot 0, TargetSpell) -- the untap-lands slot was migrated off \
                         TargetRequirement entirely"
                    );
                }
            }
        } else {
            for (has_chosen_object, targets_len) in &shapes {
                if *has_chosen_object {
                    assert_eq!(
                        *targets_len, 0,
                        "{name}: the ability carrying ChosenObject must declare ZERO \
                         TargetRequirement slots (additive migration -- both a declared \
                         target AND an untargeted choice would double-count)"
                    );
                }
            }
        }
    }
}

// ── R4: the inverse axis, frozen ─────────────────────────────────────────────

/// Unit-variant `TargetRequirement` names (serialize as bare JSON strings).
const SIMPLE_TARGET_VARIANTS: &[&str] = &[
    "TargetCreature",
    "TargetPlayer",
    "TargetPermanent",
    "TargetCreatureOrPlayer",
    "TargetAny",
    "TargetSpell",
    "TargetArtifact",
    "TargetEnchantment",
    "TargetLand",
    "TargetPlaneswalker",
    "TargetPlayerOrPlaneswalker",
    "TargetSpellOrAbilityWithSingleTarget",
    "TargetSpellWithSingleTarget",
    "TargetOpponent",
];

/// Struct/tuple-variant `TargetRequirement` names (serialize as object keys).
/// `UpToN` is handled separately (weighted by its own `count` field);
/// `TargetPermanentDistinctFrom` carries a bare `usize`, not a filter, but is
/// still a real declared slot.
const FILTER_TARGET_VARIANTS: &[&str] = &[
    "TargetCreatureWithFilter",
    "TargetPermanentWithFilter",
    "TargetSpellWithFilter",
    "TargetCardInYourGraveyard",
    "TargetCardInGraveyard",
    "TargetPermanentDistinctFrom",
];

/// Refuted rows from `pb-plan-DX28.md` §0.1 -- axis A surfaced them, adjudication
/// cleared each. Named, not counted, so a new candidate cannot silently join this
/// list by coincidence of name.
const SLOT_COUNT_REFUTED: &[&str] = &[
    // Every Equip-carrying Equipment: CR 702.6a's GRANTED ability does say
    // "target creature you control"; the printed Equip line is only the cost.
    "Batterskull",
    "Bone Saw",
    "Kite Shield",
    "Paradise Mantle",
    "Swiftfoot Boots",
    "Helm of the Host",
    "Sword of Body and Mind",
    "Sword of Feast and Famine",
    "Sword of Vengeance",
    "Umezawa's Jitte",
    // Sword of Truth and Justice's OWN Equip {2} half keeps a real
    // TargetCreatureWithFilter -- only its trigger's AddCounter migrated.
    "Sword of Truth and Justice",
    // One "target" word, TWO real slots.
    "Curtains' Call",
    "Huddle Up",
    "Victimize",
    // The trigger half genuinely prints "target"/"any target".
    "Sword of Fire and Ice",
    "Sword of Light and Shadow",
    "Sword of Sinew and Steel",
    // NOT refuted -- FOUND by this batch's own R4 census, and DEFERRED rather
    // than fixed: Concoct's half ("Surveil 3, then return a creature card
    // from your graveyard to the battlefield") prints no "target" at all and
    // is authored as a real `TargetCardInYourGraveyard`, the identical
    // OOS-DX4-6 shape `takenuma_abandoned_mire` had. This is an 18th member
    // the plan's §0.1 census did not name and this batch's own plan pinned
    // the roster at 17 -- migrating an unreviewed 18th member here would be
    // exactly the "I'll just fix one more" scope creep `memory/conventions.md`
    // warns against. Filed for a follow-up batch, not silently expanded here.
    "Connive // Concoct",
];

fn count_bare_string(v: &Value, needle: &str, parent_key: Option<&str>) -> usize {
    match v {
        Value::Object(map) => map
            .iter()
            .map(|(k, child)| count_bare_string(child, needle, Some(k.as_str())))
            .sum(),
        Value::Array(items) => items
            .iter()
            .map(|i| count_bare_string(i, needle, parent_key))
            .sum(),
        // Skip PROSE_FIELDS (a card's own free text spelling a variant name)
        // AND `inner` -- `UpToN { count, inner: Box<TargetRequirement> }` is
        // counted ONCE, as the UpToN slot itself; its wrapped requirement
        // (e.g. `TargetPermanent`) is not a SECOND slot, or "tap up to four
        // target permanents" (one UpToN, one "target" word) would
        // double-count to 2.
        Value::String(s)
            if s == needle
                && !parent_key
                    .map(|k| PROSE_FIELDS.contains(&k) || k == "inner")
                    .unwrap_or(false) =>
        {
            1
        }
        _ => 0,
    }
}

/// Same "don't double-count `UpToN`'s wrapped `inner` requirement" rule as
/// [`count_bare_string`], for the struct/tuple-variant (object-key) case --
/// `force_of_vigor.rs`'s `UpToN { inner: Box::new(TargetPermanentWithFilter(..)) }`
/// is the object-key analogue of `elder_deep_fiend.rs`'s bare-string one.
fn count_object_key(v: &Value, needle: &str, parent_key: Option<&str>) -> usize {
    match v {
        Value::Object(map) => map
            .iter()
            .map(|(k, child)| {
                let here = if k == needle && parent_key != Some("inner") {
                    1
                } else {
                    0
                };
                here + count_object_key(child, needle, Some(k.as_str()))
            })
            .sum(),
        Value::Array(items) => items
            .iter()
            .map(|i| count_object_key(i, needle, parent_key))
            .sum(),
        _ => 0,
    }
}

/// The declared `TargetRequirement` slot count over the WHOLE def (plan §0's
/// Axis A: "sum every declared `TargetRequirement` slot ... and compare with
/// the number of `\"target\"` occurrences in the combined oracle text").
fn declared_slot_count(json: &Value) -> usize {
    let mut n = 0usize;
    for variant in SIMPLE_TARGET_VARIANTS {
        n += count_bare_string(json, variant, None);
    }
    for variant in FILTER_TARGET_VARIANTS {
        n += count_object_key(json, variant, None);
    }
    // `UpToN` is ONE declared `TargetRequirement` slot in the DSL's own
    // `targets: Vec<TargetRequirement>` sense, however large its `count` --
    // "tap up to four target permanents" prints exactly ONE "target" word for
    // it (elder_deep_fiend.rs), so weighting by `count` here would make that
    // real, CR-correct target a false OOS-DX4-6 candidate. (This is the
    // opposite direction from the pre-migration Cloud of Faeries shape, which
    // this census correctly flagged not because UpToN's weight mattered but
    // because "untap up to two lands" has ZERO "target" words at all -- any
    // nonzero weight exceeds that.)
    n += find_variant_nodes(json, "UpToN").len();
    // CR 702.6a: "Equip [cost]" means "[Cost]: Attach this permanent to target
    // creature you control." The target is REAL but is never spelled "target"
    // in the printed Equip line -- it is implicit in the keyword's own rules
    // text. Every `Effect::AttachEquipment` therefore contributes one slot
    // that no oracle-text word count could ever match; excluded structurally
    // rather than by naming every Equipment card in the corpus.
    n = n.saturating_sub(find_variant_nodes(json, "AttachEquipment").len());
    n
}

#[test]
fn r4_inverse_axis_no_new_untargeted_choice_class_member() {
    let mut violations: Vec<(String, usize, usize)> = Vec::new();
    for def in all_cards() {
        if !is_effectively_complete(&def) {
            continue;
        }
        if SLOT_COUNT_REFUTED.contains(&def.name.as_str()) {
            continue;
        }
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        let slots = declared_slot_count(&json);
        // The "COMBINED oracle text" (plan §0's own phrase): a DFC's back face
        // (or an adventure/split half) has its OWN `oracle_text` field, not
        // folded into the front's -- `thaumatic_compass.rs`'s Spires of Orazca
        // back face prints "target attacking creature", invisible to
        // `def.oracle_text` alone. Case-insensitive: a sentence-initial
        // "Target creature ..." is exactly as real a target as a mid-sentence
        // "target creature ...".
        let mut combined_text = def.oracle_text.clone();
        if let Some(face) = &def.back_face {
            combined_text.push('\n');
            combined_text.push_str(&face.oracle_text);
        }
        if let Some(face) = &def.adventure_face {
            combined_text.push('\n');
            combined_text.push_str(&face.oracle_text);
        }
        let words = combined_text.to_lowercase().matches("target").count();
        if slots > words {
            violations.push((def.name.clone(), slots, words));
        }
    }
    assert!(
        violations.is_empty(),
        "R4: {} Complete def(s) declare more TargetRequirement slots than their oracle \
         text has 'target' occurrences, outside the named SLOT_COUNT_REFUTED allowlist -- \
         each is a NEW OOS-DX4-6-shaped candidate (or the allowlist needs a new refutation \
         entry): {violations:?}",
        violations.len()
    );
}

#[test]
fn r4_refuted_allowlist_entries_are_all_real_complete_defs() {
    // Non-vacuity + drift guard the OTHER direction: every name on the
    // allowlist must still exist and still be Complete, or the exclusion is
    // dead weight hiding a stale claim.
    let names: BTreeSet<String> = all_cards()
        .into_iter()
        .filter(is_effectively_complete)
        .map(|d| d.name)
        .collect();
    for n in SLOT_COUNT_REFUTED {
        assert!(
            names.contains(*n),
            "SLOT_COUNT_REFUTED names {n:?}, which is not a Complete def in the live corpus \
             -- stale entry"
        );
    }
}
