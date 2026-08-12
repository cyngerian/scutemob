//! PB-DX26 (`OOS-CARDS1-3` + `OOS-CARDS1-1`) roster sweep — **the attach surface,
//! one link earlier than CARDS-1's target-slot repair.**
//!
//! CARDS-1 (`OOS-M11-10(equip)`) closed "the picker never asks for a target" by
//! authoring CR 702.6a's `TargetRequirement` into the 17 defs that already had an
//! `AbilityDefinition::Activated { effect: Effect::AttachEquipment, .. }`. This
//! file gates the link **before** that one: `state/keyword_registry.rs`'s `K::Equip`
//! (and `K::Fortify`) arm is a `KeywordHandling::Marker` — it synthesises
//! **nothing** — so a def carrying only `AbilityDefinition::Keyword(Equip)` has no
//! equip ability at all. Not "the picker never asks"; **"there is no action to
//! pick"**.
//!
//! Everything here is derived by enumerating `mtg_engine::all_cards()` (SR-36 —
//! never grep source).
//!
//! | row | what it pins |
//! |---|---|
//! | R1 | the exact set of defs carrying the `Equip` keyword marker (21) |
//! | R2 | **every** `Equip`-marker def has a REACHABLE equip ability (violation set empty) |
//! | R3 | the deck-legal `Complete` subset of R1, pinned exact (10 -> 11) |
//! | R4 | type-line census: every `Equipment`-subtyped def, front OR back face |
//! | R5 | type-line census: every `Fortification`-subtyped def |
//! | R6 | source gate: the `Effect` nesting sites the recursive walk must cover |
//!
//! **R4/R5 are the inverse-method census** (dispatch hygiene 6 — a brief's site
//! list is a FLOOR, not a census). R1/R2 start from the *keyword marker*; R4/R5
//! start from the *printed type line* and would therefore still catch a def that
//! prints "Equip {N}" and forgot the marker as well as the ability. Two
//! independent enumerations of the same property, deliberately not sharing a
//! starting set.
//!
//! **Why the walk is recursive.** `cards1_equip_target_roster.rs` and
//! `cards1_equip_target_repair.rs:541` matched `Effect::AttachEquipment` with a
//! flat `matches!`, so a def nesting its attach inside an `Effect::Sequence`
//! dropped out of the pin **silently** (the hazard `seed-rerank-2026-08-02.md`
//! §2.7 names). `contains_attach` walks all ten `Box<Effect>` /
//! `Vec<Effect>` nesting sites in the `Effect` enum, and **R6 fails if an
//! eleventh is ever added**, so the walk cannot silently go shallow.
//!
//! **What membership asserts, and does NOT assert** (PB-DX4's `BASELINE` lesson):
//! membership means only that this def carries this specific shape. It says
//! nothing about whether the def is otherwise oracle-correct.

use mtg_engine::{
    AbilityDefinition, CardDefinition, Completeness, Effect, KeywordAbility, SubType,
};
use std::collections::BTreeSet;

// ── recursive effect walk ───────────────────────────────────────────────────────

/// Which attach effect a census row is looking for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AttachKind {
    /// `Effect::AttachEquipment` — CR 702.6a, the Equip keyword's carrier.
    Equipment,
    /// `Effect::AttachFortification` — CR 702.67a, the Fortify keyword's carrier.
    Fortification,
}

fn is_attach(effect: &Effect, kind: AttachKind) -> bool {
    match kind {
        AttachKind::Equipment => matches!(effect, Effect::AttachEquipment { .. }),
        AttachKind::Fortification => matches!(effect, Effect::AttachFortification { .. }),
    }
}

/// Recursively walk an `Effect` tree looking for the requested attach effect.
///
/// Covers every `Box<Effect>` / `Vec<Effect>` nesting site in the `Effect` enum
/// (R6 pins that this list is exhaustive). The `_ =>` arm is leaf effects, which
/// carry no nested `Effect`.
fn contains_attach(effect: &Effect, kind: AttachKind) -> bool {
    if is_attach(effect, kind) {
        return true;
    }
    match effect {
        Effect::Sequence(inner) => inner.iter().any(|e| contains_attach(e, kind)),
        Effect::Conditional {
            if_true, if_false, ..
        } => contains_attach(if_true, kind) || contains_attach(if_false, kind),
        Effect::Repeat { effect, .. } => contains_attach(effect, kind),
        Effect::ForEach { effect, .. } => contains_attach(effect, kind),
        Effect::Choose { choices, .. } => choices.iter().any(|e| contains_attach(e, kind)),
        Effect::MayPayOrElse { or_else, .. } => contains_attach(or_else, kind),
        Effect::MayPayThenEffect { then, .. } => contains_attach(then, kind),
        Effect::CoinFlip {
            on_win, on_lose, ..
        } => contains_attach(on_win, kind) || contains_attach(on_lose, kind),
        _ => false,
    }
}

/// Does this def carry an ability a player could **activate** to attach?
///
/// Activation is the only reachable route for Equip/Fortify (CR 702.6b/702.67b:
/// both are activated abilities). A `Triggered` attach (Cryptic Coat's "For
/// Mirrodin!"-style ETB self-attach) is deliberately NOT counted here — it is a
/// different mechanism with no player action, and R4 records it separately.
///
/// **`AbilityDefinition::Reconfigure` counts.** Reconfigure is the one attach
/// keyword `keyword_registry.rs` classifies as `Handled` rather than `Marker`:
/// `testing/replay_harness.rs:4049-4085` expands the variant into a real
/// `ActivatedAbility` carrying `Effect::AttachEquipment` (CR 702.151a), so Lizard
/// Blades' attach IS reachable even though its own effect tree contains no
/// `AttachEquipment` node. Counting it here rather than listing the card as a
/// residual is the honest shape: the census asks "is the attach reachable", and
/// for this one keyword the answer lives in the synth site, not the def.
fn has_activated_attach(def: &CardDefinition, kind: AttachKind) -> bool {
    let faces = std::iter::once(&def.abilities).chain(def.back_face.iter().map(|f| &f.abilities));
    faces.flatten().any(|ability| match ability {
        AbilityDefinition::Activated { effect, .. } => contains_attach(effect, kind),
        AbilityDefinition::Reconfigure { .. } => kind == AttachKind::Equipment,
        _ => false,
    })
}

/// Does this def carry a `Triggered` ability whose effect attaches? (Cryptic
/// Coat / "For Mirrodin!" — reachable, but not by a player action.)
fn has_triggered_attach(def: &CardDefinition, kind: AttachKind) -> bool {
    let faces = std::iter::once(&def.abilities).chain(def.back_face.iter().map(|f| &f.abilities));
    faces.flatten().any(|ability| match ability {
        AbilityDefinition::Triggered { effect, .. } => contains_attach(effect, kind),
        _ => false,
    })
}

fn has_keyword_marker(def: &CardDefinition, kw: &KeywordAbility) -> bool {
    let faces = std::iter::once(&def.abilities).chain(def.back_face.iter().map(|f| &f.abilities));
    faces
        .flatten()
        .any(|a| matches!(a, AbilityDefinition::Keyword(k) if k == kw))
}

fn has_subtype(def: &CardDefinition, sub: &str) -> bool {
    let want = SubType(sub.to_string());
    def.types.subtypes.contains(&want)
        || def
            .back_face
            .as_ref()
            .is_some_and(|f| f.types.subtypes.contains(&want))
}

fn names(defs: &[CardDefinition], pred: impl Fn(&CardDefinition) -> bool) -> BTreeSet<String> {
    defs.iter()
        .filter(|d| pred(d))
        .map(|d| d.name.clone())
        .collect()
}

fn pinned(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|s| s.to_string()).collect()
}

// ── R1: the Equip-marker population ─────────────────────────────────────────────

/// R1 — every def carrying `AbilityDefinition::Keyword(KeywordAbility::Equip)`,
/// pinned EXACT. This is `OOS-CARDS1-3`'s population, re-derived from
/// `all_cards()` (the seed's own figure of 21 was a grep; §2.7 of
/// `seed-rerank-2026-08-02.md` shows the same grep reads as 18 under a naive set
/// difference, because three defs mention `AttachEquipment` only inside their own
/// `Completeness::partial(..)` blocker string).
///
/// Pinned exact, not as a floor: a NEW Equipment def must fail this gate until a
/// human confirms its equip ability is authored (`pb_rs2_hybrid_phyrexian_
/// activation_roster.rs`'s reasoning).
#[test]
fn r1_equip_keyword_marker_roster_is_pinned() {
    let defs = mtg_engine::all_cards();
    let found = names(&defs, |d| has_keyword_marker(d, &KeywordAbility::Equip));
    println!(
        "R1 measured equip-marker roster ({}) = {found:?}",
        found.len()
    );
    let expected = pinned(&[
        "Blackblade Reforged",
        "Blade of the Bloodchief",
        "Bone Saw",
        "Commander's Plate",
        "Empyrial Plate",
        "Glimmer Lens",
        "Illusionist's Bracers",
        "Kite Shield",
        "Mask of Memory",
        "Paradise Mantle",
        "Sword of Body and Mind",
        "Sword of Feast and Famine",
        "Sword of Light and Shadow",
        "Sword of Sinew and Steel",
        "Sword of the Animist",
        "Sword of the Paruns",
        "Sword of Truth and Justice",
        "Sword of War and Peace",
        "The Reaver Cleaver",
        "Umbral Mantle",
        "Umezawa's Jitte",
    ]);
    assert_eq!(
        found, expected,
        "R1 (defs carrying the Equip keyword marker) has changed. If a card was ADDED, \
         confirm its equip ability is authored as an `AbilityDefinition::Activated {{ cost, \
         effect: Effect::AttachEquipment, targets: [TargetCreatureWithFilter {{ controller: \
         You }}] }}` (PB-DX26 / OOS-CARDS1-3 — a bare marker synthesises NOTHING, so the \
         printed Equip line simply does not exist at runtime) and update this pin. If \
         REMOVED, confirm that was intentional.\nFound:    {found:?}\nExpected: {expected:?}"
    );
    assert_eq!(
        found.len(),
        21,
        "non-vacuity floor: R1 must have 21 members"
    );
}

/// R2 — **the closure gate.** Every def carrying the Equip marker must also carry
/// a reachable (activated) equip ability. This set was **21 of 21 before PB-DX26**
/// and must stay empty: it is the whole defect, stated as a property rather than
/// as a name list.
#[test]
fn r2_every_equip_marker_def_has_a_reachable_equip_ability() {
    let defs = mtg_engine::all_cards();
    let violations = names(&defs, |d| {
        has_keyword_marker(d, &KeywordAbility::Equip)
            && !has_activated_attach(d, AttachKind::Equipment)
    });
    println!("R2 measured marker-without-ability set = {violations:?}");
    assert!(
        violations.is_empty(),
        "PB-DX26 / OOS-CARDS1-3: {} def(s) print 'Equip {{N}}' (they carry \
         `AbilityDefinition::Keyword(KeywordAbility::Equip)`) but have NO activated ability \
         whose effect reaches `Effect::AttachEquipment`. `keyword_registry.rs`'s `K::Equip` \
         arm is a `KeywordHandling::Marker` — it synthesises nothing — so for these defs \
         there is no ability for the provider to offer, no index for a client to name, and \
         no `Command::ActivateAbility` that could reach one. The printed ability does not \
         exist. Author it (see `skullclamp.rs` / `bone_saw.rs`).\nViolations: {violations:?}",
        violations.len()
    );
}

/// R3 — the deck-legal `Complete` subset of R1, pinned EXACT.
///
/// **10 pre-batch, 11 after.** The ten `OOS-CARDS1-3`'s rank rested on — a human
/// could legally deck any of them and simply never be offered an equip — were
/// `bone_saw`, `kite_shield`, `paradise_mantle`, `the_reaver_cleaver`,
/// `umezawas_jitte` and the five swords. Nine of those ten are `Complete` by the
/// `#[default]` derive with no `completeness` field at all (the
/// `aurelia_the_warleader` trap, this table's fourth route into it); only
/// Umezawa's Jitte declares `Completeness::Complete` explicitly.
///
/// `Sword of Body and Mind` is the eleventh and is PB-DX26's one completeness
/// FLIP UP: its `Completeness::partial(..)` note named the missing Equip {2} as
/// its *only* remaining blocker, so authoring the ability discharged it.
#[test]
fn r3_deck_legal_complete_subset_of_r1_is_pinned() {
    let defs = mtg_engine::all_cards();
    let found = names(&defs, |d| {
        has_keyword_marker(d, &KeywordAbility::Equip)
            && matches!(d.completeness, Completeness::Complete)
    });
    println!("R3 measured deck-legal Complete subset = {found:?}");
    let expected = pinned(&[
        "Bone Saw",
        "Kite Shield",
        "Paradise Mantle",
        "Sword of Body and Mind",
        "Sword of Feast and Famine",
        "Sword of Light and Shadow",
        "Sword of Sinew and Steel",
        "Sword of Truth and Justice",
        "Sword of War and Peace",
        "The Reaver Cleaver",
        "Umezawa's Jitte",
    ]);
    assert_eq!(
        found, expected,
        "R3 (deck-legal `Complete` members of the Equip-marker roster) changed. A completeness \
         FLIP is a deliberate act: re-read the def's blocker note and say in the commit why \
         every printed clause is now implemented (or why it no longer is).\nFound:    \
         {found:?}\nExpected: {expected:?}"
    );
}

// ── R4/R5: the inverse-method census (type line, not keyword) ───────────────────

/// R4 — **inverse-method census over the printed type line.** Every def whose
/// type line (front OR back face) carries the `Equipment` subtype must have a
/// reachable attach ability, by activation or by trigger. Started from the TYPE
/// LINE rather than from the keyword marker on purpose: R1/R2 cannot see a def
/// that prints "Equip {N}" and carries neither the marker nor the ability, and
/// that is exactly the shape a brief's site list would miss (dispatch hygiene 6).
///
/// The residual set below is pinned WITH REASONS, not silently excluded.
#[test]
fn r4_every_equipment_subtyped_def_has_a_reachable_attach() {
    let defs = mtg_engine::all_cards();
    let equipment = names(&defs, |d| has_subtype(d, "Equipment"));
    println!(
        "R4 measured Equipment-subtyped population ({}) = {equipment:?}",
        equipment.len()
    );

    let no_activated = names(&defs, |d| {
        has_subtype(d, "Equipment") && !has_activated_attach(d, AttachKind::Equipment)
    });
    println!("R4 measured Equipment defs with no ACTIVATED attach = {no_activated:?}");

    let no_attach_at_all = names(&defs, |d| {
        has_subtype(d, "Equipment")
            && !has_activated_attach(d, AttachKind::Equipment)
            && !has_triggered_attach(d, AttachKind::Equipment)
    });
    println!("R4 measured Equipment defs with NO attach path at all = {no_attach_at_all:?}");

    // Pinned residual — every entry needs a stated reason, re-verified at the
    // batch that pins it. See the assertion message.
    //
    // **These two are PB-DX26's inverse-census find, and R1/R2 are structurally
    // blind to them** (dispatch hygiene 6 — the brief's site list is a FLOOR).
    // Both print an "Equip {N}" line and carry NEITHER the ability NOR the
    // `KeywordAbility::Equip` marker, so the marker-derived roster the seed was
    // written from — 21 defs, `OOS-CARDS1-3` — could never see them, by either
    // the grep method or the `all_cards()` method. Only starting from the printed
    // TYPE LINE finds them. That is the whole reason R4 exists beside R2.
    //
    // Not repaired in PB-DX26, and the reason is a policy, not an oversight:
    // both defs are `Completeness::Inert` with `abilities: vec![]` — the whole
    // card is deliberately WITHHELD under the W5/W6 "no partials, no wrong game
    // state" rule, each blocked on one genuinely-absent DSL variant
    // (`EffectAmount` has no half-rounded-up for Quietus Spike; no `Condition`
    // for a combat relationship to a creature of a given subtype for Sting).
    // Authoring one clause of a withheld card is a different decision from
    // repairing a def that already ships, and it belongs to whichever batch
    // closes the real blocker. **Blast radius measured: 0 deck-legal defs** —
    // `validate_deck` rejects `Inert`, so unlike `OOS-CARDS1-3`'s ten `Complete`
    // members no human can reach either card. Filed as `OOS-DX26-1`.
    let expected_residual: BTreeSet<String> =
        pinned(&["Quietus Spike", "Sting, the Glinting Dagger"]);
    assert_eq!(
        no_attach_at_all, expected_residual,
        "R4 (inverse census, PB-DX26): an `Equipment`-subtyped def has no reachable attach \
         path — neither an `AbilityDefinition::Activated` nor an `AbilityDefinition::Triggered` \
         whose effect reaches `Effect::AttachEquipment` (walked recursively). Either author \
         the printed Equip ability, or add the def here WITH A STATED REASON (a card that is \
         an Equipment but genuinely prints no attach ability at all — e.g. one that only ever \
         attaches via another card's effect).\nFound:    {no_attach_at_all:?}\nExpected: \
         {expected_residual:?}"
    );

    // The residual is not a dumping ground: its stated reason is "the whole card
    // is deliberately withheld", and that reason is itself checked. A def that
    // stops being `Inert` while still having no attach path stops qualifying for
    // the excusal and fails here — the `KNOWN_FALSE_OFFERS` lesson from SIM-2
    // ("an excusal list is a debt register with a maturity date"), enforced rather
    // than written down.
    let by_name: std::collections::HashMap<&str, &CardDefinition> =
        defs.iter().map(|d| (d.name.as_str(), d)).collect();
    for name in &expected_residual {
        let def = by_name
            .get(name.as_str())
            .unwrap_or_else(|| panic!("R4 residual member '{name}' must exist in all_cards()"));
        assert!(
            matches!(def.completeness, Completeness::Inert(_)),
            "R4 residual member '{name}' is excused from the attach census ONLY because the \
             whole card is deliberately withheld (`Completeness::Inert`, `abilities: vec![]`, \
             W5/W6 'no partials, no wrong game state'). It is now {:?}, so the excusal no \
             longer applies: author its printed Equip line (CR 702.6a) or restate the \
             residual with a new reason.",
            def.completeness
        );
        assert!(
            def.abilities.is_empty(),
            "R4 residual member '{name}' now carries abilities, so it is no longer the \
             withheld-whole-card case the residual excuses. Author its equip ability."
        );
    }

    assert!(
        equipment.len() >= 30,
        "non-vacuity floor: the `Equipment` subtype walk found only {} defs — the field \
         access or the subtype string is broken (this is a real, populated corpus)",
        equipment.len()
    );
}

/// R5 — the same inverse census for `Fortification` (CR 702.67a, `OOS-CARDS1-1`).
/// One def in the corpus, and it had the identical never-asks defect equip had.
#[test]
fn r5_every_fortification_subtyped_def_has_a_reachable_attach() {
    let defs = mtg_engine::all_cards();
    let fortifications = names(&defs, |d| has_subtype(d, "Fortification"));
    println!(
        "R5 measured Fortification-subtyped population ({}) = {fortifications:?}",
        fortifications.len()
    );
    assert_eq!(
        fortifications,
        pinned(&["Darksteel Garrison"]),
        "R5: the `Fortification` population changed. A new Fortification must declare its \
         printed Fortify ability as an `AbilityDefinition::Activated {{ effect: \
         Effect::AttachFortification, targets: [TargetPermanentWithFilter(Land + controller \
         You)] }}` — CR 702.67a is 'attach to target LAND you control', so copying the equip \
         repair's `TargetCreatureWithFilter` verbatim would be wrong.\nFound: {fortifications:?}"
    );

    let no_attach = names(&defs, |d| {
        has_subtype(d, "Fortification")
            && !has_activated_attach(d, AttachKind::Fortification)
            && !has_triggered_attach(d, AttachKind::Fortification)
    });
    assert!(
        no_attach.is_empty(),
        "R5 (inverse census): a `Fortification`-subtyped def has no reachable attach path: \
         {no_attach:?}"
    );

    let marker_without_ability = names(&defs, |d| {
        has_keyword_marker(d, &KeywordAbility::Fortify)
            && !has_activated_attach(d, AttachKind::Fortification)
    });
    assert!(
        marker_without_ability.is_empty(),
        "R5: a def carries `KeywordAbility::Fortify` (a `KeywordHandling::Marker` that \
         synthesises nothing) with no activated `Effect::AttachFortification` ability: \
         {marker_without_ability:?}"
    );
}

// ── R6: the recursion the census depends on cannot silently go shallow ──────────

/// R6 — source gate on `contains_attach`'s coverage.
///
/// The recursive walk above enumerates the `Effect` enum's nesting sites by hand.
/// If a new `Box<Effect>` / `Vec<Effect>` field is ever added to `Effect`, the
/// walk goes shallow **silently** and every census in this file quietly narrows —
/// the exact failure mode `seed-rerank-2026-08-02.md` §2.7 names about the flat
/// `matches!` this file replaces. This gate counts the nesting sites in the enum's
/// own source and fails when the count moves, so the walk cannot rot unnoticed.
///
/// **Stated residual**: this is a source count, not a type-level proof. It cannot
/// see a nesting site expressed some other way (e.g. `Option<Box<Effect>>`, or a
/// newtype wrapping `Effect`) — those forms do not exist in the enum today, and
/// the count below is over the two forms that do. An exhaustive `match` with no
/// wildcard would be the stronger construction, but `Effect` has ~150 variants and
/// the arms would be noise; this gate buys the same alarm for two dozen lines.
#[test]
fn r6_effect_nesting_sites_are_pinned() {
    let src = include_str!("../../../card-types/src/cards/card_definition.rs");
    let start = src
        .find("pub enum Effect {")
        .expect("the `Effect` enum must exist in card_definition.rs");
    let mut depth = 0usize;
    let mut end = start;
    for (i, ch) in src[start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    assert!(
        end > start,
        "failed to bracket-match the `Effect` enum body"
    );
    let body = &src[start..end];

    // Comments describe nesting without being nesting. Strip them, exactly as
    // `pb_dx24_trigger_zone_roster.rs` does — PB-DX32's `OOS-DX32-6` proved a
    // commented-out row can leave the compiled artefact while a source gate stays
    // green, so the stripping is load-bearing, not cosmetic.
    let code_only: String = body
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    let boxed = code_only.matches("Box<Effect>").count();
    let vecs = code_only.matches("Vec<Effect>").count();
    println!("R6 measured Effect nesting sites: Box<Effect>={boxed}, Vec<Effect>={vecs}");
    assert_eq!(
        (boxed, vecs),
        (8, 2),
        "The `Effect` enum's nesting sites changed (found Box<Effect>={boxed}, \
         Vec<Effect>={vecs}; expected 8 and 2). `contains_attach` in this file walks them by \
         hand: Conditional{{if_true,if_false}}, Repeat{{effect}}, ForEach{{effect}}, \
         MayPayOrElse{{or_else}}, MayPayThenEffect{{then}}, CoinFlip{{on_win,on_lose}} \
         (Box) and Sequence(..), Choose{{choices}} (Vec). Add the new site to \
         `contains_attach` and re-pin this count, or every attach census in this file \
         silently stops seeing effects nested inside it."
    );
}
