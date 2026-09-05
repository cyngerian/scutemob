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
