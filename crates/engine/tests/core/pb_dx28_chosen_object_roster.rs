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
//!   SLOT-SHAPED `"target"` word count, outside a named allowlist of the
//!   REFUTED rows in `pb-plan-DX28.md` §0.1.
//! * **R5** — every `ChosenObject` in the corpus sits in an effect arm the
//!   pre-pass actually walks. Added by the `/review` cycle, which defeated
//!   R1–R4 by moving one to an unsupported arm and staying green.
//!
//! **What R4 does NOT promise.** An earlier draft of this doc said "this is the
//! census, frozen, so the class cannot silently regrow". That is stronger than
//! the row supports and the claim is withdrawn. R4 is a SUBTRACTION of two
//! counts, so any `"target"` word that is not a slot declaration cancels a real
//! slot — the `/review` defeated it with a single sentence ("becomes the target
//! of a spell") in the SAME ability. `slot_shaped_target_words` now strips the
//! known non-slot idioms, which raises precision and does not change the shape
//! of the argument: an idiom not on that list still cancels, and
//! `declared_slot_count`'s `AttachEquipment` subtraction is a second, structural
//! cancellation channel that can mask a real slot on any Equipment. R4 is a
//! ratchet against the KNOWN population regrowing, not a proof that the class is
//! empty. The honest statement of the bound is `OOS-DX28-8`.
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

/// The exact 18 corpus defs naming `EffectTarget::ChosenObject`, by name.
///
/// **18, not the plan's 17, and the difference is the point.** `pb-plan-DX28.md`
/// §0.1's census enumerated 17 by comparing declared target-requirement SLOTS
/// against `"target"` occurrences in the printed oracle text. `Connive // Concoct`
/// was invisible to it and was found by **R4** below — this file's own inverse
/// axis — after the roster had already been written down. It is migrated, not
/// deferred: `OOS-DX4-6` is CLOSED by this batch, and closing a class while a
/// known deck-legal `Complete` member keeps the old shape would close it on a
/// false premise. AC 6448's "registry member lists are FLOORS" does not stop
/// applying at the number a plan happened to write.
const CHOSEN_OBJECT_MEMBERS: &[&str] = &[
    "Azorius Chancery",
    "Boros Garrison",
    "Cloud of Faeries",
    "Connive // Concoct",
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
        18,
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
    //
    // PB-DX57 (`OOS-DX28-1`): the list is now the named
    // `UNSUPPORTED_UNTARGETED_AXES` const, DERIVED-CHECKED by
    // `r2b_unsupported_untargeted_axes_are_derived_from_the_matcher` below. As an
    // inline literal it was invisible to every enumeration of this file's variant
    // lists, and a THIRD unimplemented `TargetFilter` field would simply not have
    // been here -- with this loop still green.
    for f in &filters {
        for unsupported in UNSUPPORTED_UNTARGETED_AXES {
            let is_set = f.get(*unsupported).is_some_and(|v| !v.is_null());
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

/// Every target-declaring `AbilityDefinition` node's OWN `targets` list, as
/// parsed JSON arrays, paired with whether that SAME node also contains a
/// `ChosenObject`. Returns `(has_chosen_object, targets_len)` per node.
///
/// **The variant list is the whole correctness of this row**, and it has been
/// wrong twice. `Activated` is here because `takenuma_abandoned_mire`'s Channel
/// ability is one, not a Triggered or a Spell. `Fuse` is here because
/// `Connive // Concoct`'s Concoct half is one — a split card's half declares its
/// own `targets` and is neither a `Spell` nor an `Activated` node, so the
/// three-variant version of this walk reported "R1 found a ChosenObject but no
/// node carries it" and could not tell a migration it could not SEE from a
/// migration that had not happened. That is the `seed-rerank-2026-08-02.md` §2.7
/// hazard (a flat/short match dropping a nesting site in silence) in a gate
/// written by the batch that cites it.
///
/// **And PB-DX28's widening to six was ITSELF short by two, which is why the
/// list is gone.** `AbilityDefinition::Aftermath` declares `targets` and was
/// never listed; `AbilityDefinition::Splice` GAINED a `targets` field one batch
/// later (PB-DX18, `OOS-M11-5`, CR 702.47a) and nothing reddened. Both are
/// corpus-observed, not theoretical — see
/// `pb_dx57_ability_target_variants::d3`'s printed axis-2 set. So the six went
/// stale within one batch of being widened, by the ordinary act of authoring a
/// rule, and this row reported success for a fortnight while blind to two
/// variants. That is `OOS-DX28-5` measured rather than predicted.
///
/// The hand-written list is therefore replaced by
/// `pb_dx57_ability_target_variants::target_declaring_ability_variants()`, which
/// DERIVES the set from `pub enum AbilityDefinition`'s own declaration. A walk
/// that needs this question must call that, never re-type a list here.
fn ability_target_shapes(def: &mtg_engine::CardDefinition) -> Vec<(bool, usize)> {
    let json = serde_json::to_value(def).expect("CardDefinition serializes");
    let mut out = Vec::new();
    for variant in crate::pb_dx57_ability_target_variants::target_declaring_ability_variants() {
        for node in find_variant_nodes(&json, &variant) {
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
///
/// **PB-DX57 (`OOS-DX28-1`) — this list was STALE and the repair below is LIVE, not
/// just a pin.** `TargetSpellOrAbility` (`card_definition.rs:3127`) is a unit variant
/// and was in NEITHER this list nor [`FILTER_TARGET_VARIANTS`], so the three lists
/// together covered **21 of the 22** declared variants. Every def declaring it was
/// under-counted by one slot in R4's `slots > words` SUBTRACTION — which means a real
/// over-declaration on such a def cancels to zero and is never reported, on a row whose
/// whole mechanism the module doc already calls out as a subtraction. Live exposure at
/// HEAD is one `Complete` deck-legal def, `deflecting_swat` (1 slot, 1 slot-shaped
/// `"target"` word, so it does not trip R4 either before or after — the under-count was
/// real and its cancellation happened not to be load-bearing on today's corpus).
/// [`r4b_target_requirement_variant_lists_partition_the_declaration`] is what stops the
/// 22nd from going missing again.
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
    // Added by PB-DX57 (`OOS-DX28-1`). Missing since the list was written; see the
    // doc above for what it silently under-counted.
    "TargetSpellOrAbility",
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

/// The `TargetRequirement` variants counted by NODE rather than by name/key —
/// `UpToN { count, inner }`, whose weight is deliberately 1 (see
/// [`declared_slot_count`]).
///
/// A `const` rather than the inline `"UpToN"` string literal it replaces, so that
/// [`r4b_target_requirement_variant_lists_partition_the_declaration`] pins the
/// spelling this file actually walks with. An inline literal is invisible to any
/// enumeration of this file's variant lists, which is `OOS-DX28-5`'s own shape.
const WEIGHTED_TARGET_VARIANTS: &[&str] = &["UpToN"];

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
    // PB-DX18 (`OOS-M11-5`), CR 702.47a: TWO slots, ONE printed "target", and the
    // duplication is correct rather than a candidate.
    //
    // Glacial Ray prints "Glacial Ray deals 2 damage to any target" ONCE, and declares
    // that requirement TWICE: once on its `AbilityDefinition::Spell` (its own cast) and
    // once on its `AbilityDefinition::Splice` (the copy of the same sentence that a
    // spliced spell gains — *"copy this card's text box onto that spell"*). Only one of
    // the two is ever live for a given cast, so this is not the OOS-DX4-6 shape (a slot
    // with no printed "target" behind it); it is the same printed sentence reachable
    // through two mutually exclusive routes. This axis counts DECLARATIONS against
    // printed WORDS and cannot see that distinction — stated here rather than tuned away,
    // because a splice def that really did over-declare would still be caught by the
    // count moving past 2.
    "Glacial Ray",
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
    for variant in WEIGHTED_TARGET_VARIANTS {
        n += find_variant_nodes(json, variant).len();
    }
    // CR 702.6a: "Equip [cost]" means "[Cost]: Attach this permanent to target
    // creature you control." The target is REAL but is never spelled "target"
    // in the printed Equip line -- it is implicit in the keyword's own rules
    // text. Every `Effect::AttachEquipment` therefore contributes one slot
    // that no oracle-text word count could ever match; excluded structurally
    // rather than by naming every Equipment card in the corpus.
    n = n.saturating_sub(find_variant_nodes(json, "AttachEquipment").len());
    n
}

/// The number of `"target"` occurrences in `text` that could plausibly be a
/// DECLARED TARGET SLOT, i.e. the "…N target creature…" shape.
///
/// **Added by the PB-DX28 `/review` cycle (finding 2, MEDIUM), which defeated
/// R4 by execution.** The reviewer planted a `Complete` def printing *"Whenever
/// this creature becomes the target of a spell, return a creature you control to
/// its owner's hand."* — a genuine `OOS-DX4-6` member, authored as a real
/// `TargetCreatureWithFilter` — and R4 stayed GREEN, because the bare
/// `matches("target")` count was 1 and the slot count was 1, so the arithmetic
/// cancelled. Deleting only the offsetting phrase turned R4 red, which is what
/// proves the cancellation was the whole mechanism.
///
/// The cancelling word does **not** have to be in a different ability, which is
/// what `OOS-DX28-8` originally said; a single sentence can supply both. Roughly
/// 39 corpus defs carry `"becomes the target of"` / `"can't be the target of"` /
/// a bare `"targets"`, so the exposure is real rather than theoretical.
///
/// The fix is to strip the idioms in which `"target"` is NOT a slot declaration
/// before counting. This is a precision improvement, not a proof: an idiom not
/// on this list still cancels, and the list is a human judgement per phrase with
/// nothing behind it — the same standing caveat `filter_states_a_quality`'s
/// exclusion list carries. `OOS-DX28-8` records the residual.
fn slot_shaped_target_words(text: &str) -> usize {
    let mut t = text.to_lowercase();
    // Order matters: the longer forms first, so a shorter one cannot consume
    // half of a longer one and leave a fragment that still matches "target".
    for idiom in [
        "becomes the target of",
        "become the target of",
        "becomes a target of",
        "can't be the target of",
        "cannot be the target of",
        "can't become the target of",
        "the target of",
        "a target of",
        "new target",
        "change the target",
        "changes the target",
    ] {
        t = t.replace(idiom, " ");
    }
    // A standalone "targets" is a VERB ("whenever a spell targets …") or the
    // plural noun ("its targets"), never the "N target X" slot shape — which is
    // always singular. Word-bounded so "targets" inside a longer token is left
    // alone.
    let mut out = 0usize;
    for word in t.split(|c: char| !c.is_ascii_alphabetic()) {
        if word == "target" {
            out += 1;
        }
    }
    out
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
        let words = slot_shaped_target_words(&combined_text);
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

// ── R5: every ChosenObject sits in an effect arm the pre-pass supports ───────

/// The `Effect` arms `effects::resolve_pending_object_choices` actually walks.
///
/// **This list is the whole safety of the channel.** `EffectTarget::ChosenObject`
/// is banked by a pre-pass that matches only these arms; reaching
/// `resolve_effect_target_list_indexed` with nothing banked resolves to the EMPTY
/// set behind a `debug_assert!`, which is compiled OUT in release. So a
/// `ChosenObject` authored into any other arm is a silent no-op in a release
/// build — the effect simply does nothing, with no panic and no diagnostic.
const SUPPORTED_ARMS: &[&str] = &["MoveZone", "AddCounter", "UntapPermanent"];

/// Count occurrences of `key` as a map key anywhere in `v`.
fn count_key_occurrences(v: &Value, key: &str) -> usize {
    match v {
        Value::Object(map) => map
            .iter()
            .map(|(k, child)| usize::from(k == key) + count_key_occurrences(child, key))
            .sum(),
        Value::Array(items) => items.iter().map(|i| count_key_occurrences(i, key)).sum(),
        _ => 0,
    }
}

/// **Added by the PB-DX28 `/review` cycle (finding 1, MEDIUM), which defeated
/// R1–R4 by execution.**
///
/// The reviewer changed `frantic_search.rs`'s `Effect::UntapPermanent` to
/// `Effect::TapPermanent`, keeping the identical `ChosenObject` value — a real,
/// silently-broken migration — and **all five existing rows stayed green**. R1
/// pins by def NAME, so an arm change inside an existing member is invisible to
/// it; R2 inspects filter axes; R3 inspects `targets.is_empty()`. The only thing
/// in the whole workspace that noticed was the runtime `debug_assert!` firing
/// incidentally inside a fuzz test that happened to cast that card — which is
/// not a gate, and is absent from a release build entirely.
///
/// The plan's §1.4 claim that R3 makes "a 19th use redden so the author must
/// confirm the arm is supported" was therefore true of a 19th *def* and false of
/// a 19th *use* or an arm change within an existing one. This row closes that
/// gap, in the shape R2 already uses for filter axes: assert the count of
/// `ChosenObject` nodes reachable inside supported arms equals the corpus-wide
/// count.
///
/// **Stated residual**: this is subtree containment, so a supported arm that
/// NESTED an unsupported one would mask it. None of the three has a nested
/// `Effect` field (`MoveZone { target, to, controller_override }`,
/// `AddCounter { target, counter, count }`, `UntapPermanent { target }` — read
/// from the enum, not assumed), so the two readings coincide today. If a
/// supported arm ever gains a nested `Effect`, this row needs a path-aware walk.
#[test]
fn r5_every_chosen_object_sits_in_a_supported_effect_arm() {
    let mut total = 0usize;
    let mut supported = 0usize;
    let mut offenders: Vec<String> = Vec::new();

    for def in all_cards() {
        if !def_contains_variant(&def, "ChosenObject") {
            continue;
        }
        let json = serde_json::to_value(&def).expect("CardDefinition serializes");
        let def_total = count_key_occurrences(&json, "ChosenObject");
        let def_supported: usize = SUPPORTED_ARMS
            .iter()
            .flat_map(|arm| find_variant_nodes(&json, arm))
            .map(|node| count_key_occurrences(node, "ChosenObject"))
            .sum();
        if def_supported != def_total {
            offenders.push(format!(
                "{}: {def_total} ChosenObject node(s), only {def_supported} inside {SUPPORTED_ARMS:?}",
                def.name
            ));
        }
        total += def_total;
        supported += def_supported;
    }

    assert!(
        offenders.is_empty(),
        "a ChosenObject is authored in an effect arm `resolve_pending_object_choices` does \
         NOT walk. It will resolve to the EMPTY set in release, silently -- the debug_assert \
         that catches it in a debug build is compiled out. Either add the arm to the pre-pass \
         (and to SUPPORTED_ARMS here) or author the effect differently.\n  {}",
        offenders.join("\n  ")
    );
    // Non-vacuity floors, in BOTH directions: the row must be counting something,
    // and the two counts must be equal for a reason other than both being zero.
    assert!(
        total >= 18,
        "non-vacuity: the corpus carries at least one ChosenObject per migrated member \
         (>= 18 members, several naming it twice); counted {total}"
    );
    assert_eq!(
        supported, total,
        "the per-def loop and the running totals must agree"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PB-DX57 (`OOS-DX28-1`) — the three hand-maintained lists in this file, pinned
// ─────────────────────────────────────────────────────────────────────────────
//
// `OOS-DX28-1` is the CLASS of hand-maintained structural fingerprints that go
// silently blind the moment their subject grows a member. This file held four of
// them: R3's `AbilityDefinition` variant walk (repaired separately, and now
// derived by `pb_dx57_ability_target_variants`), R4's two `TargetRequirement`
// lists plus the inline `"UpToN"`, R5's `SUPPORTED_ARMS`, and R2's inline list of
// `TargetFilter` axes the untargeted matcher does not implement. The three below
// are the remaining three, and **two of them were already stale when censused**.

use crate::pb_dx45_may_pay_roster::{function_body, match_arms, EFFECTS_MOD_RS};
use crate::pb_dx57_declared_source::{
    declared_enum_variants, declared_struct_fields, CARD_DEFINITION_RS,
};

/// The `TargetFilter` axes `effects::filter_matches_object_untargeted` refuses
/// outright rather than narrowing by (its fail-closed prefix guard).
///
/// Extracted from `r2`'s inline literal by PB-DX57 so it can be pinned; see
/// [`r2b_unsupported_untargeted_axes_are_derived_from_the_matcher`].
const UNSUPPORTED_UNTARGETED_AXES: &[&str] = &["max_cmc_amount", "min_cmc_amount"];

/// **Census row 2+3 — a PARTITION, and the repair is LIVE.**
///
/// [`declared_slot_count`] sums three disjoint lists —
/// [`SIMPLE_TARGET_VARIANTS`] (bare-string unit variants),
/// [`FILTER_TARGET_VARIANTS`] (object-key payload variants) and
/// [`WEIGHTED_TARGET_VARIANTS`] (`UpToN`, counted by node). Together they must be
/// **exactly** `pub enum TargetRequirement`'s declared variants: a variant in none
/// of the three contributes ZERO to `declared_slot_count`, and because R4 is a
/// SUBTRACTION (`slots > words`), a missing variant does not merely under-report —
/// it silently CANCELS a real over-declaration elsewhere in the same def.
///
/// **What this found.** `TargetSpellOrAbility` was in none of the three, so the
/// lists covered 21 of 22 and had done since they were written. Repaired above.
///
/// Set equality, not a subset, and not a count: a count assertion passes when one
/// variant leaves and another joins, which is `OOS-DX20b-5`'s lesson one axis over.
///
/// **Revert to watch red**: remove any one name from any of the three lists.
#[test]
fn r4b_target_requirement_variant_lists_partition_the_declaration() {
    let simple: BTreeSet<String> = SIMPLE_TARGET_VARIANTS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let filtered: BTreeSet<String> = FILTER_TARGET_VARIANTS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let weighted: BTreeSet<String> = WEIGHTED_TARGET_VARIANTS
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    // Disjointness first: a variant counted by TWO of the three lists would be
    // double-counted by `declared_slot_count`, which is the same defect in the
    // opposite direction and is invisible to a union-only check.
    for (a, an, b, bn) in [
        (&simple, "SIMPLE", &filtered, "FILTER"),
        (&simple, "SIMPLE", &weighted, "WEIGHTED"),
        (&filtered, "FILTER", &weighted, "WEIGHTED"),
    ] {
        let both: Vec<&String> = a.intersection(b).collect();
        assert!(
            both.is_empty(),
            "{an}_TARGET_VARIANTS and {bn}_TARGET_VARIANTS both name {both:?}; \
             declared_slot_count would count it twice"
        );
    }

    let mut covered: BTreeSet<String> = BTreeSet::new();
    covered.extend(simple.iter().cloned());
    covered.extend(filtered.iter().cloned());
    covered.extend(weighted.iter().cloned());

    let declared = declared_enum_variants(CARD_DEFINITION_RS, "TargetRequirement");
    println!(
        "PB-DX57 row 2+3: {} declared TargetRequirement variants, {} covered by \
         declared_slot_count's three lists",
        declared.len(),
        covered.len()
    );

    assert_eq!(
        covered,
        declared,
        "declared_slot_count's three variant lists no longer partition \
         `pub enum TargetRequirement`. A declared variant in NONE of them contributes \
         zero slots, and because R4 subtracts word count from slot count, that does not \
         merely under-report -- it CANCELS a real over-declaration on the same def and \
         R4 stays green. (That is exactly how `TargetSpellOrAbility` sat outside all \
         three lists for the whole of PB-DX28's life.) \
         declared-but-uncovered = {:?}, covered-but-undeclared = {:?}",
        declared.difference(&covered).collect::<Vec<_>>(),
        covered.difference(&declared).collect::<Vec<_>>()
    );
}

/// **Census row 4 — `SUPPORTED_ARMS`, derived from the pre-pass it describes.**
///
/// [`SUPPORTED_ARMS`]'s own doc says *"this list is the whole safety of the
/// channel"*: a `ChosenObject` in an arm `effects::resolve_pending_object_choices`
/// does not walk resolves to the EMPTY set in release, behind a `debug_assert!`
/// that is compiled out. `r5` above compares the corpus against that list — so if
/// the PRE-PASS ever stops walking an arm while the list still names it, `r5`
/// stays green while the effect silently does nothing. Nothing in the workspace
/// derived the list from the function.
///
/// Pinned against the FUNCTION, not against `pub enum Effect`: the semantic set is
/// the pre-pass's arms, and comparing it to the 106-variant enum would be wrong in
/// both directions. The subset check against the declaration is a second, weaker
/// leg that catches a variant RENAME (which would make `find_variant_nodes` match
/// nothing and take `r5` vacuous).
///
/// **Revert to watch red**: delete the `Effect::UntapPermanent { target } => target,`
/// arm from `resolve_pending_object_choices`. It still compiles — the `_ => return
/// true` wildcard absorbs it — `r5` stays GREEN, and only this row notices.
#[test]
fn r5b_supported_arms_are_derived_from_the_pre_pass() {
    let arms = match_arms(
        EFFECTS_MOD_RS,
        "resolve_pending_object_choices",
        "match effect {",
        "Effect",
        8,
    );
    let derived: BTreeSet<String> = arms.iter().flat_map(|a| a.names.iter().cloned()).collect();
    let pinned: BTreeSet<String> = SUPPORTED_ARMS.iter().map(|s| (*s).to_string()).collect();

    println!(
        "PB-DX57 row 4: resolve_pending_object_choices walks {derived:?} \
         ({} arm group(s) parsed)",
        arms.len()
    );

    assert_eq!(
        derived, pinned,
        "SUPPORTED_ARMS no longer matches the arms `effects::resolve_pending_object_choices` \
         actually walks. If the pre-pass LOST an arm this list still names, every \
         `EffectTarget::ChosenObject` authored into it resolves to the empty set in a \
         RELEASE build with no panic and no diagnostic, and r5 above stays green because \
         it reads this list rather than the function. If the pre-pass GAINED one, r5 \
         under-covers."
    );

    let declared = declared_enum_variants(CARD_DEFINITION_RS, "Effect");
    assert!(
        derived.is_subset(&declared),
        "the pre-pass names {:?}, which `pub enum Effect` does not declare -- a rename \
         would also make `find_variant_nodes` match nothing and take r5 vacuous",
        derived.difference(&declared).collect::<Vec<_>>()
    );
}

// ── Census row 19: the axes the untargeted matcher does not implement ────────

/// The binding name of the `&TargetFilter` parameter of `fn <fn_name>`, read off
/// the signature rather than assumed to be `filter`.
fn target_filter_binding(rel: &str, fn_name: &str) -> String {
    let raw = crate::pb_dx45_may_pay_roster::strip_comments_preserving_length(
        &crate::pb_dx57_declared_source::read_workspace_file(rel),
    );
    let at = raw
        .find(&format!("fn {fn_name}("))
        .unwrap_or_else(|| panic!("`fn {fn_name}(` not found in {rel}"));
    let close = raw[at..]
        .find(')')
        .map(|i| i + at)
        .unwrap_or_else(|| panic!("`fn {fn_name}`'s parameter list is never closed"));
    let sig = &raw[at..close];
    let needle = ": &TargetFilter";
    let hit = sig.find(needle).unwrap_or_else(|| {
        panic!(
            "`fn {fn_name}` in {rel} has no `&TargetFilter` parameter -- this pin is \
                pointed at the wrong function"
        )
    });
    sig[..hit]
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// Every `<binding>.<field>` read in `body`.
fn filter_field_reads(body: &str, binding: &str) -> BTreeSet<String> {
    let needle = format!("{binding}.");
    let bytes = body.as_bytes();
    let mut out = BTreeSet::new();
    let mut from = 0usize;
    while let Some(i) = body[from..].find(&needle) {
        let at = from + i;
        let prev_ok = at == 0 || !(bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_');
        let start = at + needle.len();
        let field: String = body[start..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if prev_ok && !field.is_empty() && field.starts_with(|c: char| c.is_ascii_lowercase()) {
            out.insert(field);
        }
        from = at + needle.len();
    }
    out
}

/// Every function `body` calls with `binding` among the arguments, i.e. the
/// helpers a filter is handed off to.
///
/// DERIVED rather than hand-listed, because a hand-listed callee set would itself
/// be an `OOS-DX28-1` member: the day someone factors a fourth axis out into a new
/// helper, a fixed list of three stops seeing it and this whole row goes quietly
/// short.
fn filter_callees(body: &str, binding: &str, self_name: &str) -> BTreeSet<String> {
    let b = body.as_bytes();
    let mut out = BTreeSet::new();
    for (i, _) in body.match_indices('(') {
        // The identifier immediately before the `(`.
        let name: String = body[..i]
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if name.is_empty() || !name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_') {
            continue;
        }
        if ["if", "match", "for", "while", "return", "fn"].contains(&name.as_str())
            || name == self_name
        {
            continue;
        }
        // Balance the argument list.
        let mut depth = 0usize;
        let mut end = None;
        for (k, ch) in body[i..].char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i + k);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(end) = end else { continue };
        let args = &body[i + 1..end];
        // Word-bounded `binding` among the arguments.
        let mut from = 0usize;
        let mut names_it = false;
        while let Some(rel) = args[from..].find(binding) {
            let at = from + rel;
            let abs = i + 1 + at;
            let before_ok = abs == 0 || !(b[abs - 1].is_ascii_alphanumeric() || b[abs - 1] == b'_');
            let after = abs + binding.len();
            let after_ok =
                after >= b.len() || !(b[after].is_ascii_alphanumeric() || b[after] == b'_');
            if before_ok && after_ok {
                names_it = true;
                break;
            }
            from = at + binding.len();
        }
        if names_it {
            out.insert(name);
        }
    }
    out
}

/// **Census row 19 — two-sided, and the two sides catch opposite failures.**
///
/// `r2` above asserts that no corpus `ChosenObject` filter sets an axis
/// `effects::filter_matches_object_untargeted` cannot honour. Its list of such
/// axes was an inline literal naming two fields of a **33-field** struct, and
/// nothing compared it to anything. A NEW `TargetFilter` field that the untargeted
/// matcher also does not implement is silently outside the list, so a corpus def
/// setting it is mishandled in production — *"rejected outright, not narrowed"*, in
/// `r2`'s own words — while `r2` stays green. That is `OOS-DX28-1` exactly.
///
/// Two legs:
///
/// 1. **Nothing is unimplemented in silence.** Every declared `TargetFilter` field
///    must be read somewhere in the matcher's own call graph — the function itself
///    plus every helper it hands the filter to, the callee set DERIVED from the
///    body rather than hand-listed. A field read nowhere means the untargeted
///    channel neither honours it nor refuses it: it is ignored, which is the
///    over-wide answer space `r2`'s doc says this must prevent.
/// 2. **The refusal list is the matcher's own.** The fields named in the matcher's
///    fail-closed prefix guard must be exactly [`UNSUPPORTED_UNTARGETED_AXES`].
///
/// **The guard region is the function body up to its FIRST `return false;`.** That
/// is sound only because the fail-closed guard is the function's first statement —
/// a property this row therefore also enforces, since moving it changes the
/// extracted set and reddens leg 2. Stated because it is an assumption, not a fact
/// about Rust.
///
/// **Residual, stated rather than discovered later.** Leg 1 proves each field is
/// *read*, not that it is read *correctly*; and the callee walk is depth-1, which
/// is checked rather than assumed (the assertion below requires each callee to hand
/// the filter to nobody else).
///
/// **Revert to watch red**: add a field to `pub struct TargetFilter` and implement
/// it nowhere (leg 1), or add a third disjunct to the matcher's opening guard
/// without listing it above (leg 2).
#[test]
fn r2b_unsupported_untargeted_axes_are_derived_from_the_matcher() {
    const MATCHER: &str = "filter_matches_object_untargeted";
    let binding = target_filter_binding(EFFECTS_MOD_RS, MATCHER);
    assert_eq!(
        binding, "filter",
        "the matcher's &TargetFilter parameter is now named {binding:?}; the derivation \
         below reads it off the signature, so this is informational -- but every doc in \
         this file spells it `filter`"
    );

    let body = function_body(EFFECTS_MOD_RS, MATCHER);

    // ── leg 2: the fail-closed prefix guard ──────────────────────────────────
    let guard_end = body.find("return false;").unwrap_or_else(|| {
        panic!(
            "`{MATCHER}` no longer contains a `return false;` -- its fail-closed guard \
                is gone, and every corpus filter setting an unimplemented axis is now \
                silently ignored rather than refused"
        )
    });
    let guard = &body[..guard_end];
    assert!(
        guard.len() < 600 && !guard.contains("matches_filter("),
        "the extracted fail-closed guard is {} bytes and/or already reaches the \
         `matches_filter` hand-off, so it is not the function's opening statement any \
         more. This pin assumes the guard comes FIRST; re-establish that or re-point the \
         extraction.",
        guard.len()
    );
    let guard_fields = filter_field_reads(guard, &binding);
    let pinned: BTreeSet<String> = UNSUPPORTED_UNTARGETED_AXES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    assert_eq!(
        guard_fields, pinned,
        "UNSUPPORTED_UNTARGETED_AXES no longer matches the axes \
         `{MATCHER}` refuses outright in its opening guard. r2 above walks THIS list, so \
         a third unimplemented axis missing from it means a corpus filter setting it is \
         rejected outright in production with no test saying so."
    );

    // ── leg 1: every declared field is read somewhere in the call graph ──────
    let callees = filter_callees(&body, &binding, MATCHER);
    assert!(
        callees.len() >= 3,
        "non-vacuity: `{MATCHER}` hands its filter to {callees:?} (measured 3 at HEAD: \
         matches_filter, check_has_counter_type, check_chosen_subtype_filter). A derivation \
         that finds none would make leg 1 compare the declaration against this function's \
         eleven direct reads and fail for the wrong reason."
    );

    let mut read: BTreeSet<String> = filter_field_reads(&body, &binding);
    for callee in &callees {
        let cb = function_body(EFFECTS_MOD_RS, callee);
        let cbind = target_filter_binding(EFFECTS_MOD_RS, callee);
        read.extend(filter_field_reads(&cb, &cbind));
        // Depth-1 is CHECKED, not assumed: if a helper hands the filter on again,
        // this walk stops one level short and leg 1 silently under-reads.
        let deeper = filter_callees(&cb, &cbind, callee);
        assert!(
            deeper.is_empty(),
            "`{callee}` hands the filter on to {deeper:?}, so the depth-1 call-graph walk \
             this row performs is one level short and leg 1 would under-read. Extend the \
             walk (or state why the deeper reads cannot matter)."
        );
    }

    let declared = declared_struct_fields(CARD_DEFINITION_RS, "TargetFilter");
    println!(
        "PB-DX57 row 19: TargetFilter declares {} field(s); the untargeted matcher's \
         call graph ({callees:?}) reads {}; refused outright: {guard_fields:?}",
        declared.len(),
        read.len()
    );

    let unread: Vec<&String> = declared.difference(&read).collect();
    assert!(
        unread.is_empty(),
        "`pub struct TargetFilter` declares {unread:?}, which NOTHING in \
         `{MATCHER}`'s call graph reads. The untargeted channel therefore neither honours \
         that axis nor refuses it -- it IGNORES it, so a ChosenObject filter setting it \
         gets a silently over-wide answer space, with r2 above green. Either implement it \
         (in `matches_filter` or a helper) or add it to the fail-closed opening guard AND \
         to UNSUPPORTED_UNTARGETED_AXES."
    );
    assert!(
        pinned.is_subset(&declared),
        "UNSUPPORTED_UNTARGETED_AXES names {:?}, which TargetFilter does not declare",
        pinned.difference(&declared).collect::<Vec<_>>()
    );
}
