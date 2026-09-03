//! PB-DX20b (`OOS-DX20-10`, HIGH; `OOS-DX20-5`) — the printed Enchant line is an OR over
//! card **types**, and `EnchantFilter` could not say so.
//!
//! CR 702.5a: *"Enchant [object or player]"* restricts both what an Aura spell may **target**
//! (CR 303.4a) and what the permanent may remain **attached to** (CR 303.4c / 704.5m).
//! `imprisoned_in_the_moon` — `Complete`, deck-legal — prints *"Enchant creature, land, or
//! planeswalker"* and declared `EnchantTarget::Permanent`, whose `matches_enchant_target` arm
//! is a bare `true`. So the cast gate and the CR 704.5m SBA both accepted an **artifact** or an
//! **enchantment**, and PB-DX20 (which made the offer layer enumerate the requirement) turned
//! that into something a human can click.
//!
//! `EnchantFilter` had `has_card_type` (exactly ONE type) and `has_subtypes` (an OR over
//! **sub**types) and no OR over card **types**. PB-DX20b added `has_card_types: Vec<CardType>`,
//! lowered onto `TargetFilter.has_card_types`, which already existed.
//!
//! ## What each row is for
//!
//! * `t1`/`t2`/`t3` — the three printed-legal classes are **accepted** (CR 303.4a).
//! * `t4`/`t5` — artifact and enchantment are **refused**. These two are `OOS-DX20-10`'s live
//!   defect: at the merge base both were **accepted**.
//! * `t6`/`t6b` — CR 704.5m, the load-bearing SBA half, with its **control**. An attachment
//!   made illegal without moving the Aura detaches; one that stays legal does not. A
//!   detach-everything bug must not pass `t6`, which is what `t6b` exists to stop.
//! * `t7` — `kayas_ghostform`, `OOS-DX20-5`: the controller half (an opponent's creature is
//!   refused) **and** the narrowing half (a planeswalker you control is now accepted; at the
//!   merge base the def declared `EnchantTarget::Creature` and refused it).
//! * `t8` — `breath_of_fury`, the stage-0 census find no seed row and no memo cell names:
//!   a printed *"you control"* that was simply dropped.
//! * `t9` — the two arithmetics agree, measured END TO END.
//!
//! ## `t9` is a CONSISTENCY pin, and that is stated rather than implied
//!
//! PB-DX20's own durable lesson is that *a differential probe between two consumers of one
//! function proves consistency, not correctness*. `t9` is exactly that probe, so `t1`-`t8` are
//! the **correctness** half of this file and `t9` is the structural half.
//!
//! That is not a caveat borrowed from a memo — it was **executed**. Deleting
//! `has_card_types: f.has_card_types.clone()` from `casting::enchant_filter_to_target_filter`
//! (revert row R2) leaves `t9` **GREEN** and reddens `t4`/`t5`/`t6`: one wrong lowering makes
//! the offer, the cast and the SBA all wrong in the same direction, so the consistency probe
//! agrees perfectly while the engine accepts an artifact. Conversely, deleting the
//! `matches_filter` call from `sba::enchant_filter_matches` (R3) makes the two sides disagree
//! and `t9` goes red where `t4`/`t5` stay green. Neither half covers the other.
//!
//! ## Two findings from the revert matrix that a later batch should read before "simplifying"
//!
//! 1. **The brief for this file asked `t4`/`t5` to assert a refusal message naming the Enchant
//!    restriction, and that message is structurally unreachable.** `casting.rs`'s CR 303.4a
//!    gate is *"a DELIBERATELY REDUNDANT second check"* (its own words); PB-DX20 synthesizes
//!    the requirement upstream, so the refusal actually emitted is
//!    `InvalidTarget("declared 1 target(s) but 1 could not be matched to a requirement slot")`.
//!    See `assert_refused_by_the_enchant_restriction` for what is asserted instead.
//! 2. **That gate is nevertheless load-bearing in one direction only, and the pair of reverts
//!    proves which.** With `matches_filter` gone from the SBA predicate (R3), `t4`/`t5` still
//!    refuse — the requirement rejects upstream — so the gate adds nothing in the *accepting*
//!    direction. With the gate's `Filtered` arm forced to `false` (R10), `t1`/`t2`/`t3` redden
//!    — targets the requirement had accepted — so it is decisive in the *refusing* direction.
//!    Anyone tempted to delete the gate as "already covered upstream" should read R3 and R10
//!    together in `memory/primitives/pb-DX20b-execution-notes.md`.
//!
//! `t9` is written end to end — offer, cast and SBA on real boards — rather than by calling
//! the two functions directly, because **both are `pub(crate)`**:
//! `casting::enchant_filter_to_target_filter` and `sba::matches_enchant_target` are not
//! reachable from an integration test, and this file deliberately does **not** widen a
//! visibility in `src/` to make a test compile. The public surfaces it uses instead are
//! `queries::spell_target_requirements` (which returns the lowering's own output verbatim,
//! since `enchant_target_to_requirement`'s `Filtered` arm is
//! `TargetPermanentWithFilter(enchant_filter_to_target_filter(f))`),
//! `queries::legal_targets_per_slot`, `effects::matches_filter`, `process_command` and
//! `start_game`. That is a *stronger* probe than the direct call would have been: it measures
//! the three surfaces a client actually meets.
//!
//! ## Fixtures
//!
//! Every probe about a real card uses the **real card def**, through
//! `enrich_spec_from_def(ObjectSpec::card(..).with_card_id(card_name_to_id(..)), &defs)`. A
//! hand-built stand-in would defeat the whole file: what is under test is the **declared**
//! `EnchantTarget` in `crates/card-defs/src/defs/`, and a stand-in re-declares it.
//! `t9`'s matrix is the one place hand-built Auras are correct, because its subject is the
//! `EnchantFilter` **shape space**, not any card.
//!
//! `GameStateBuilder::build()` registers no static continuous effects (`OOS-DX43-6`), so
//! `imprisoned_in_the_moon`'s own Layer-4 *"is a colorless land"* effect does **not** apply on
//! these boards. That is load-bearing for `t6`: if it did apply, the enchanted artifact would
//! become a Land and stay legal.

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    all_cards, card_name_to_id, effects, enrich_spec_from_def, legal_targets_per_slot,
    process_command, spell_target_requirements, start_game, CardDefinition, CardRegistry, CardType,
    Command, EnchantControllerConstraint, EnchantFilter, EnchantTarget, GameEvent, GameState,
    GameStateBuilder, GameStateError, KeywordAbility, ManaPool, ObjectId, ObjectSpec, PlayerId,
    Step, SubType, SuperType, Target, TargetController, TargetRequirement, ZoneId,
};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn registry() -> Arc<CardRegistry> {
    CardRegistry::new(all_cards())
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// The REAL card def, enriched — never a hand-built stand-in. See the module doc.
fn real_card(owner: PlayerId, name: &str, zone: ZoneId) -> ObjectSpec {
    let defs = defs();
    assert!(
        defs.contains_key(name),
        "PB-DX20b fixture: `{}` is not in `all_cards()`; this file's probes are about the \
         DECLARED EnchantTarget in crates/card-defs, so a missing def must fail loudly rather \
         than silently degrade to a naked ObjectSpec (`enrich_spec_from_def` returns its input \
         unchanged for an unknown name)",
        name
    );
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .with_card_id(card_name_to_id(name))
            .in_zone(zone),
        &defs,
    )
}

/// A `CastSpellData` naming exactly `targets` for `card`, cast by `player`.
fn cast(player: PlayerId, card: ObjectId, targets: Vec<Target>) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
        targets,
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: None,
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        face_down_kind: None,
        additional_costs: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }))
}

/// Generous mana — every probe here is about target legality, never about affordability.
fn plenty() -> ManaPool {
    ManaPool {
        white: 5,
        blue: 5,
        black: 5,
        red: 5,
        green: 5,
        colorless: 5,
        ..Default::default()
    }
}

/// A planeswalker that is not a creature and not a land.
fn planeswalker(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::planeswalker(owner, name, 4)
}

/// A basic land carrying a basic land subtype (CR 205.3i / 205.4a).
fn basic_land(owner: PlayerId, name: &str, subtype: &str) -> ObjectSpec {
    ObjectSpec::land(owner, name)
        .with_supertypes(vec![SuperType::Basic])
        .with_subtypes(vec![SubType(subtype.to_string())])
}

// ─────────────────────────────────────────────────────────────────────────────
// The `imprisoned_in_the_moon` cast board — t1..t5
// ─────────────────────────────────────────────────────────────────────────────

/// p1 holds `imprisoned_in_the_moon` and controls one permanent of every class the
/// `EnchantTarget::Permanent` declaration used to admit. p2 exists so `matches_enchant_target`'s
/// controller arm has something to be `Any` about.
fn imprisoned_board() -> (GameState, PlayerId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .player_mana(p1, plenty())
        .object(real_card(p1, "Imprisoned in the Moon", ZoneId::Hand(p1)))
        .object(ObjectSpec::creature(p1, "Board Creature", 2, 2))
        .object(basic_land(p1, "Board Land", "Island"))
        .object(planeswalker(p1, "Board Planeswalker"))
        .object(ObjectSpec::artifact(p1, "Board Artifact"))
        .object(ObjectSpec::enchantment(p1, "Board Enchantment"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("imprisoned board builds");

    let aura = find_object(&state, "Imprisoned in the Moon");
    (state, p1, aura)
}

/// Cast `imprisoned_in_the_moon` at the named board permanent.
fn cast_imprisoned_at(target_name: &str) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let (state, p1, aura) = imprisoned_board();
    let target = find_object(&state, target_name);
    process_command(state, cast(p1, aura, vec![Target::Object(target)]))
}

#[test]
/// CR 702.5a / 303.4a — "Enchant creature, land, or planeswalker" accepts a **creature**.
fn t1_imprisoned_accepts_a_creature() {
    let result = cast_imprisoned_at("Board Creature");
    assert!(
        result.is_ok(),
        "CR 303.4a: Imprisoned in the Moon prints \"Enchant creature, land, or planeswalker\" \
         and must accept a creature; got {:?}",
        result.err()
    );
}

#[test]
/// CR 702.5a / 303.4a — "Enchant creature, land, or planeswalker" accepts a **land**.
fn t2_imprisoned_accepts_a_land() {
    let result = cast_imprisoned_at("Board Land");
    assert!(
        result.is_ok(),
        "CR 303.4a: Imprisoned in the Moon must accept a land; got {:?}",
        result.err()
    );
}

#[test]
/// CR 702.5a / 303.4a — "Enchant creature, land, or planeswalker" accepts a **planeswalker**.
fn t3_imprisoned_accepts_a_planeswalker() {
    let result = cast_imprisoned_at("Board Planeswalker");
    assert!(
        result.is_ok(),
        "CR 303.4a: Imprisoned in the Moon must accept a planeswalker; got {:?}",
        result.err()
    );
}

/// The refusal shape shared by `t4` and `t5`.
///
/// ## The brief asked for a message naming the Enchant restriction, and that message is
/// ## structurally unreachable — reported rather than asserted anyway
///
/// `casting.rs`'s CR 303.4a gate does carry
/// `InvalidTarget("target does not match Enchant restriction (…)")`, but it is a
/// **deliberately redundant second check** (its own block comment says so): PB-DX20 synthesizes
/// the announceable `TargetRequirement` upstream, so `validate_targets_inner` refuses the
/// declaration first, at CR 601.2c slot assignment, with
/// `InvalidTarget("declared 1 target(s) but 1 could not be matched to a requirement slot")`.
/// Measured, not assumed — that is the verbatim string these two probes receive.
///
/// So a needle on the word "Enchant" would be a needle on a message the engine never emits
/// here, i.e. a test that passes for a reason its author did not intend. What is asserted
/// instead is *the same claim, keyed on the mechanism*: the refusal is `InvalidTarget` (never a
/// cost or step refusal — the board hands p1 five of every colour precisely so a shortfall
/// cannot masquerade as a legality refusal), **and** the offer layer excludes this permanent
/// from the Aura's target slot while including a printed-legal one from the SAME board. A
/// refusal the picker predicted is a target-legality refusal; a refusal the picker did not
/// predict is `SR-38`'s clean-offer-then-guaranteed-422 defect, which this file would then be
/// reporting rather than passing.
fn assert_refused_by_the_enchant_restriction(target_name: &str) {
    let result = cast_imprisoned_at(target_name);
    let err = match result {
        Ok(_) => panic!(
            "OOS-DX20-10: Imprisoned in the Moon prints \"Enchant creature, land, or \
             planeswalker\" and must NOT accept `{}`. This is the live HIGH: at the merge base \
             the def declared EnchantTarget::Permanent, whose matches_enchant_target arm is a \
             bare `true`, and PB-DX20 made the widened offer human-reachable.",
            target_name
        ),
        Err(e) => e,
    };
    assert!(
        matches!(err, GameStateError::InvalidTarget(_)),
        "CR 303.4a: expected GameStateError::InvalidTarget for `{}` — a different variant \
         means the cast failed for a reason other than target legality; got {:?}",
        target_name,
        err
    );

    // The mechanism half: the offer layer agrees, and it discriminates.
    let (state, p1, aura) = imprisoned_board();
    let refused = find_object(&state, target_name);
    let allowed = find_object(&state, "Board Creature");
    let reqs = spell_target_requirements(&state, aura, &[], None, false);
    let offered = legal_targets_per_slot(&state, p1, aura, &reqs);
    let slot = offered
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("CR 303.4a: an Aura spell has exactly one target slot"));
    assert!(
        !slot.contains(&Target::Object(refused)),
        "SR-38 / OOS-DX20-10: `{}` is not one of Imprisoned in the Moon's three printed \
         classes, so the offer layer must not enumerate it. If it does, the refusal above is a \
         clean offer followed by a guaranteed refusal. Slot: {:?}",
        target_name,
        slot
    );
    assert!(
        slot.contains(&Target::Object(allowed)),
        "Non-vacuity floor: the offer slot must still contain the printed-legal creature — an \
         EMPTY slot would satisfy the assertion above while measuring nothing. Slot: {:?}",
        slot
    );
}

#[test]
/// CR 702.5a / 303.4a — **`OOS-DX20-10`'s live defect.** An artifact is not one of the three
/// printed classes, and at the merge base this cast was ACCEPTED.
fn t4_imprisoned_refuses_an_artifact() {
    assert_refused_by_the_enchant_restriction("Board Artifact");
}

#[test]
/// CR 702.5a / 303.4a — **`OOS-DX20-10`'s live defect**, second class. An enchantment is not
/// one of the three printed classes, and at the merge base this cast was ACCEPTED.
fn t5_imprisoned_refuses_an_enchantment() {
    assert_refused_by_the_enchant_restriction("Board Enchantment");
}

// ─────────────────────────────────────────────────────────────────────────────
// t6 / t6b — CR 704.5m, and its control
// ─────────────────────────────────────────────────────────────────────────────

/// Put `imprisoned_in_the_moon` on the battlefield **already attached** to `victim`, then run
/// the game's SBA sweep.
///
/// The Aura is never moved: the illegality is produced by what the enchanted permanent IS, not
/// by where the Aura sits. That is CR 704.5m's own shape (*"an Aura is attached to an illegal
/// object … its controller puts it into its owner's graveyard"*), and it is why `t6` and `t6b`
/// differ in exactly one argument.
///
/// The victim is passed in whole rather than built from a bare `CardType`, and the reason is a
/// bug this fixture's first draft had: `ObjectSpec::card(..).with_types([Creature])` is a
/// creature with **no toughness**, which CR 704.5f destroys on the same sweep — so `t6b`'s
/// control reported "the Aura fell off a creature" when what had happened was that the creature
/// died first. A control that fails for the wrong reason is worse than no control.
fn attached_board(victim: ObjectSpec) -> (GameState, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);

    let victim = victim.in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .object(real_card(p1, "Imprisoned in the Moon", ZoneId::Battlefield))
        .object(victim)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("attached board builds");

    let aura = find_object(&state, "Imprisoned in the Moon");
    let victim_id = find_object(&state, "Attach Victim");

    if let Some(aura_obj) = state.objects_mut().get_mut(&aura) {
        aura_obj.attached_to = Some(victim_id);
    }
    if let Some(victim_obj) = state.objects_mut().get_mut(&victim_id) {
        victim_obj.attachments.push_back(aura);
    }
    (state, aura)
}

/// Did the CR 704.5m SBA detach the Aura and put it into its owner's graveyard?
fn aura_fell_off(state: GameState, aura: ObjectId) -> (bool, GameState) {
    let (after, events) = start_game(state).expect("start_game succeeds");
    let fired = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if *object_id == aura));
    (fired, after)
}

#[test]
/// CR 704.5m / 303.4c — an `imprisoned_in_the_moon` attached to an **artifact** is illegally
/// attached and is put into its owner's graveyard.
///
/// This is the SBA half of `OOS-DX20-10`, and it is the half a cast-path-only fix would have
/// left open: an Aura can arrive on the battlefield attached without ever being cast (Auras put
/// onto the battlefield attached, control changes, type changes), so CR 303.4a and CR 704.5m are
/// two independent doors on the same restriction.
fn t6_imprisoned_falls_off_an_artifact() {
    let (state, aura) = attached_board(ObjectSpec::artifact(p(1), "Attach Victim"));
    let (fell, after) = aura_fell_off(state, aura);
    assert!(
        fell,
        "CR 704.5m: Imprisoned in the Moon attached to an artifact is illegally attached \
         (\"Enchant creature, land, or planeswalker\") and must be put into its owner's \
         graveyard"
    );
    // CR 400.7: the detached Aura is a NEW object in the graveyard, so the battlefield
    // `ObjectId` the event carries is dead and cannot be looked up. Assert on the destination
    // ZONE instead — the event alone says the Aura came off, never where it landed.
    let in_graveyard = after.objects().iter().any(|(_, o)| {
        o.characteristics.name == "Imprisoned in the Moon" && o.zone == ZoneId::Graveyard(p(1))
    });
    assert!(
        in_graveyard,
        "CR 704.5m: the detached Aura goes to its OWNER's graveyard. Objects at rest: {:?}",
        after
            .objects()
            .iter()
            .map(|(_, o)| (o.characteristics.name.clone(), o.zone))
            .collect::<Vec<_>>()
    );
}

#[test]
/// CR 704.5m — **the control.** An `imprisoned_in_the_moon` attached to a **creature** is
/// legally attached and must NOT be detached.
///
/// Without this row a "detach every Aura" regression passes `t6`. It is written as a sibling
/// rather than folded into `t6` so the failure message names which half broke.
fn t6b_imprisoned_stays_on_a_creature() {
    let (state, aura) = attached_board(ObjectSpec::creature(p(1), "Attach Victim", 2, 2));
    let (fell, after) = aura_fell_off(state, aura);
    assert!(
        !fell,
        "CR 704.5m: a creature IS one of Imprisoned in the Moon's three printed classes; \
         detaching it would mean the fix detaches everything, which `t6` alone cannot \
         distinguish from a correct fix"
    );
    let aura_obj = after
        .objects()
        .get(&aura)
        .expect("the Aura object is still present");
    assert_eq!(
        aura_obj.zone,
        ZoneId::Battlefield,
        "CR 704.5m: a legally attached Aura stays on the battlefield"
    );
    assert!(
        aura_obj.attached_to.is_some(),
        "CR 704.5m: a legally attached Aura keeps its attachment"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t7 — kayas_ghostform (`OOS-DX20-5`), both halves
// ─────────────────────────────────────────────────────────────────────────────

/// p1 holds `kayas_ghostform`; p1 and p2 each control a creature and a planeswalker.
fn ghostform_board() -> (GameState, PlayerId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry())
        .player_mana(p1, plenty())
        .object(real_card(p1, "Kaya's Ghostform", ZoneId::Hand(p1)))
        .object(ObjectSpec::creature(p1, "My Creature", 2, 2))
        .object(ObjectSpec::creature(p2, "Their Creature", 2, 2))
        .object(planeswalker(p1, "My Planeswalker"))
        .object(planeswalker(p2, "Their Planeswalker"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("ghostform board builds");

    let aura = find_object(&state, "Kaya's Ghostform");
    (state, p1, aura)
}

fn cast_ghostform_at(target_name: &str) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let (state, p1, aura) = ghostform_board();
    let target = find_object(&state, target_name);
    process_command(state, cast(p1, aura, vec![Target::Object(target)]))
}

#[test]
/// CR 702.5a / 303.4a — `OOS-DX20-5`, **both halves in one test**, because the seed is one
/// defect with two directions and splitting them would let either half pass alone.
///
/// `kayas_ghostform` prints *"Enchant creature or planeswalker you control"* and declared
/// `EnchantTarget::Creature`:
///
/// * the **controller** half — "you control" was dropped entirely, so an OPPONENT's creature
///   was a legal target and a legal attachment;
/// * the **narrowing** half — "or planeswalker" was dropped, so a planeswalker you control was
///   REFUSED at the merge base.
///
/// A fix that only widened, or only added the controller clause, satisfies one assertion here
/// and fails the other.
fn t7_ghostform_is_your_creature_or_your_planeswalker_and_nothing_else() {
    // Half 1, the narrowing: a planeswalker you control is legal. REFUSED at the merge base.
    let pw = cast_ghostform_at("My Planeswalker");
    assert!(
        pw.is_ok(),
        "OOS-DX20-5 (narrowing half): Kaya's Ghostform prints \"Enchant creature or \
         planeswalker you control\" and must accept a planeswalker you control; got {:?}",
        pw.err()
    );

    // A creature you control is legal — the case the merge-base declaration got right, kept
    // so a fix cannot pass by refusing everything.
    let own = cast_ghostform_at("My Creature");
    assert!(
        own.is_ok(),
        "CR 303.4a: a creature you control is legal for Kaya's Ghostform; got {:?}",
        own.err()
    );

    // Half 2, the controller clause: an opponent's creature is NOT legal. ACCEPTED at the
    // merge base, because `EnchantTarget::Creature` carries no controller constraint at all.
    let theirs = cast_ghostform_at("Their Creature");
    assert!(
        matches!(theirs, Err(GameStateError::InvalidTarget(_))),
        "OOS-DX20-5 (controller half): \"you control\" was dropped by the merge-base \
         declaration, so an opponent's creature was a legal target; got {:?}",
        theirs.map(|_| "Ok")
    );

    // …and the two halves must not be satisfiable independently: an opponent's PLANESWALKER
    // is refused too, which a fix that widened without adding the controller clause would
    // accept.
    let their_pw = cast_ghostform_at("Their Planeswalker");
    assert!(
        matches!(their_pw, Err(GameStateError::InvalidTarget(_))),
        "OOS-DX20-5: widening to `has_card_types` WITHOUT `controller: You` accepts an \
         opponent's planeswalker; got {:?}",
        their_pw.map(|_| "Ok")
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t8 — breath_of_fury, the stage-0 census find
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// CR 702.5a / 303.4a — `breath_of_fury` prints *"Enchant creature you control"* and declared
/// `EnchantTarget::Creature`, dropping the controller clause.
///
/// **No seed row and no v4 memo cell names this def**; it is a stage-0 census find, and it
/// needed no new expressiveness at all — `EnchantFilter.controller` has existed since PB-DX20.
/// It is in this file because a census that finds a member and does not gate it has not
/// measured anything.
fn t8_breath_of_fury_is_your_creature_only() {
    let p1 = p(1);
    let p2 = p(2);

    let build = || {
        GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry())
            .player_mana(p1, plenty())
            .object(real_card(p1, "Breath of Fury", ZoneId::Hand(p1)))
            .object(ObjectSpec::creature(p1, "My Creature", 2, 2))
            .object(ObjectSpec::creature(p2, "Their Creature", 2, 2))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .expect("breath of fury board builds")
    };

    let state = build();
    let aura = find_object(&state, "Breath of Fury");
    let mine = find_object(&state, "My Creature");
    let own = process_command(state, cast(p1, aura, vec![Target::Object(mine)]));
    assert!(
        own.is_ok(),
        "CR 303.4a: Breath of Fury must accept a creature you control; got {:?}",
        own.err()
    );

    let state = build();
    let aura = find_object(&state, "Breath of Fury");
    let theirs = find_object(&state, "Their Creature");
    let opp = process_command(state, cast(p1, aura, vec![Target::Object(theirs)]));
    assert!(
        matches!(opp, Err(GameStateError::InvalidTarget(_))),
        "CR 702.5a: Breath of Fury prints \"Enchant creature you control\"; the merge-base \
         `EnchantTarget::Creature` dropped \"you control\" and accepted an opponent's \
         creature; got {:?}",
        opp.map(|_| "Ok")
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t9 — the two arithmetics agree, END TO END
// ─────────────────────────────────────────────────────────────────────────────

/// One `EnchantFilter` under test, with a label for the failure message.
struct FilterCase {
    label: &'static str,
    filter: EnchantFilter,
}

/// One candidate permanent under test.
struct ShapeCase {
    name: &'static str,
    /// `true` = controlled by the Aura's controller (p1); `false` = by p2.
    mine: bool,
    spec: fn(PlayerId, &'static str) -> ObjectSpec,
}

fn filter_cases() -> Vec<FilterCase> {
    let ct = |t: CardType| Some(t);
    vec![
        FilterCase {
            label: "no restriction (bare default)",
            filter: EnchantFilter::default(),
        },
        FilterCase {
            label: "has_card_types [Creature, Land, Planeswalker] (imprisoned_in_the_moon)",
            filter: EnchantFilter {
                has_card_types: vec![CardType::Creature, CardType::Land, CardType::Planeswalker],
                ..Default::default()
            },
        },
        FilterCase {
            label: "has_card_types [Creature, Planeswalker] + You (kayas_ghostform)",
            filter: EnchantFilter {
                has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                controller: EnchantControllerConstraint::You,
                ..Default::default()
            },
        },
        FilterCase {
            label: "has_card_type Creature + You (breath_of_fury)",
            filter: EnchantFilter {
                has_card_type: ct(CardType::Creature),
                controller: EnchantControllerConstraint::You,
                ..Default::default()
            },
        },
        FilterCase {
            label: "has_card_type Creature + Opponent",
            filter: EnchantFilter {
                has_card_type: ct(CardType::Creature),
                controller: EnchantControllerConstraint::Opponent,
                ..Default::default()
            },
        },
        FilterCase {
            label: "has_card_type Land + has_subtype Mountain (awaken_the_ancient)",
            filter: EnchantFilter {
                has_card_type: ct(CardType::Land),
                has_subtype: Some(SubType("Mountain".to_string())),
                ..Default::default()
            },
        },
        FilterCase {
            label: "has_card_type Land + basic + You (ossification / dimensional_exile)",
            filter: EnchantFilter {
                has_card_type: ct(CardType::Land),
                basic: true,
                controller: EnchantControllerConstraint::You,
                ..Default::default()
            },
        },
        FilterCase {
            label: "has_card_type Land + nonbasic",
            filter: EnchantFilter {
                has_card_type: ct(CardType::Land),
                nonbasic: true,
                ..Default::default()
            },
        },
        FilterCase {
            label: "has_card_type Land + has_subtypes [Forest, Plains]",
            filter: EnchantFilter {
                has_card_type: ct(CardType::Land),
                has_subtypes: vec![SubType("Forest".to_string()), SubType("Plains".to_string())],
                ..Default::default()
            },
        },
        FilterCase {
            // The two card-type conjuncts are INDEPENDENT and AND together — the field's own
            // doc says so. This row is the only place that claim is executed.
            label: "has_card_type Land AND has_card_types [Creature, Land] (both conjuncts)",
            filter: EnchantFilter {
                has_card_type: ct(CardType::Land),
                has_card_types: vec![CardType::Creature, CardType::Land],
                ..Default::default()
            },
        },
        FilterCase {
            label: "has_card_types [Artifact] (single-member OR)",
            filter: EnchantFilter {
                has_card_types: vec![CardType::Artifact],
                ..Default::default()
            },
        },
    ]
}

fn shape_cases() -> Vec<ShapeCase> {
    vec![
        ShapeCase {
            name: "own creature",
            mine: true,
            spec: |o, n| ObjectSpec::creature(o, n, 2, 2),
        },
        ShapeCase {
            name: "opp creature",
            mine: false,
            spec: |o, n| ObjectSpec::creature(o, n, 2, 2),
        },
        ShapeCase {
            name: "own basic Mountain",
            mine: true,
            spec: |o, n| basic_land(o, n, "Mountain"),
        },
        ShapeCase {
            name: "opp basic Mountain",
            mine: false,
            spec: |o, n| basic_land(o, n, "Mountain"),
        },
        ShapeCase {
            name: "own basic Forest",
            mine: true,
            spec: |o, n| basic_land(o, n, "Forest"),
        },
        ShapeCase {
            name: "own nonbasic Gate land",
            mine: true,
            spec: |o, n| ObjectSpec::land(o, n).with_subtypes(vec![SubType("Gate".to_string())]),
        },
        ShapeCase {
            name: "own planeswalker",
            mine: true,
            spec: |o, n| ObjectSpec::planeswalker(o, n, 4),
        },
        ShapeCase {
            name: "opp planeswalker",
            mine: false,
            spec: |o, n| ObjectSpec::planeswalker(o, n, 4),
        },
        ShapeCase {
            name: "own artifact",
            mine: true,
            spec: ObjectSpec::artifact,
        },
        ShapeCase {
            name: "own enchantment",
            mine: true,
            spec: ObjectSpec::enchantment,
        },
        ShapeCase {
            name: "own artifact creature",
            mine: true,
            spec: |o, n| {
                ObjectSpec::creature(o, n, 1, 1)
                    .with_types(vec![CardType::Artifact, CardType::Creature])
            },
        },
        ShapeCase {
            name: "own animated basic Mountain (Land + Creature)",
            mine: true,
            spec: |o, n| {
                basic_land(o, n, "Mountain").with_types(vec![CardType::Land, CardType::Creature])
            },
        },
    ]
}

/// A synthetic Aura carrying `Enchant(Filtered(f))`, with a zero mana cost so a cast can never
/// fail for affordability.
///
/// This is the ONE place in this file a hand-built Aura is correct: `t9`'s subject is the
/// `EnchantFilter` shape space, not any card. Every other probe uses the real def, because its
/// subject is what the def DECLARES.
fn synthetic_aura(owner: PlayerId, f: &EnchantFilter, zone: ZoneId) -> ObjectSpec {
    ObjectSpec::enchantment(owner, "T9 Aura")
        .with_subtypes(vec![SubType("Aura".to_string())])
        .with_keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(f.clone())))
        .in_zone(zone)
}

fn t9_cast_board(
    f: &EnchantFilter,
    shape: &ShapeCase,
) -> (GameState, PlayerId, ObjectId, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let owner = if shape.mine { p1 } else { p2 };

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .player_mana(p1, plenty())
        .object(synthetic_aura(p1, f, ZoneId::Hand(p1)))
        .object((shape.spec)(owner, "T9 Candidate"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("t9 cast board builds");

    let aura = find_object(&state, "T9 Aura");
    let candidate = find_object(&state, "T9 Candidate");
    (state, p1, aura, candidate)
}

fn t9_attached_board(f: &EnchantFilter, shape: &ShapeCase) -> (GameState, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let owner = if shape.mine { p1 } else { p2 };

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(synthetic_aura(p1, f, ZoneId::Battlefield))
        .object((shape.spec)(owner, "T9 Candidate"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("t9 attached board builds");

    let aura = find_object(&state, "T9 Aura");
    let candidate = find_object(&state, "T9 Candidate");
    if let Some(a) = state.objects_mut().get_mut(&aura) {
        a.attached_to = Some(candidate);
    }
    if let Some(c) = state.objects_mut().get_mut(&candidate) {
        c.attachments.push_back(aura);
    }
    (state, aura)
}

#[test]
/// CR 702.5a / 303.4a / 704.5m — **the two arithmetics agree**, over the whole
/// `EnchantFilter` × object-shape matrix, measured END TO END.
///
/// PB-DX20 left two independent copies of the "what does an `EnchantFilter` field mean"
/// arithmetic: the cast/offer lowering (`casting::enchant_filter_to_target_filter`, reached
/// from `handle_cast_spell` and from `queries::spell_target_requirements`) and a hand-rolled
/// six-field predicate in `sba::enchant_filter_matches` (reached from the CR 303.4a gate and
/// the CR 704.5m SBA). PB-DX20b deleted the second and made it call the first. This row is the
/// executed proof, across **four** surfaces per matrix cell:
///
/// 1. the OFFER — `legal_targets_per_slot` over `spell_target_requirements`' answer;
/// 2. the LOWERING's characteristic half — `effects::matches_filter` against the very
///    `TargetFilter` the query returned, plus the controller clause read off `tf.controller`;
/// 3. the CAST — `Command::CastSpell` accepted or refused;
/// 4. the SBA — CR 704.5m, on a board where the same pair is already attached.
///
/// **This is a CONSISTENCY pin, not a correctness pin** (PB-DX20's own durable lesson: a
/// differential probe between two consumers of one function proves consistency, not
/// correctness). One wrong lowering satisfies all four. `t1`-`t8` are the correctness half.
///
/// Both functions under test are `pub(crate)` and are deliberately NOT made public for a
/// test's convenience — see the module doc for the public surfaces used instead.
fn t9_offer_lowering_cast_and_sba_agree_across_the_filter_matrix() {
    let filters = filter_cases();
    let shapes = shape_cases();
    let mut cells = 0usize;
    let mut legal_cells = 0usize;

    for fc in &filters {
        for shape in &shapes {
            cells += 1;

            // ── Surface 1 + 2: the offer, and the lowering the offer was built from.
            let (state, p1, aura, candidate) = t9_cast_board(&fc.filter, shape);
            let reqs = spell_target_requirements(&state, aura, &[], None, false);
            assert_eq!(
                reqs.len(),
                1,
                "CR 303.4a: an Aura spell announces exactly one target; [{}] x [{}] produced \
                 {:?}",
                fc.label,
                shape.name,
                reqs
            );
            let tf = match &reqs[0] {
                TargetRequirement::TargetPermanentWithFilter(tf) => tf.clone(),
                other => panic!(
                    "PB-DX20b: `EnchantTarget::Filtered` must lower to \
                     TargetPermanentWithFilter — that IS the lowering under test; [{}] x [{}] \
                     produced {:?}",
                    fc.label, shape.name, other
                ),
            };

            let offered = legal_targets_per_slot(&state, p1, aura, &reqs);
            let offer_says_legal = offered
                .first()
                .is_some_and(|slot| slot.contains(&Target::Object(candidate)));

            let chars = mtg_engine::calculate_characteristics(&state, candidate)
                .expect("the candidate is on the battlefield and has characteristics");
            let candidate_controller = state
                .objects()
                .get(&candidate)
                .map(|o| o.controller)
                .expect("the candidate exists");
            let controller_ok = match tf.controller {
                TargetController::Any => true,
                TargetController::You => candidate_controller == p1,
                TargetController::Opponent => candidate_controller != p1,
                // Written out rather than wildcarded on purpose. `EnchantControllerConstraint`
                // has exactly three variants and the lowering maps them 1:1, so this arm is
                // unreachable *by construction* — and a fourth `TargetController` variant
                // arriving here would mean the lowering had started producing something an
                // `EnchantFilter` cannot express, which is a finding rather than a default.
                other => panic!(
                    "PB-DX20b t9: `enchant_filter_to_target_filter` produced \
                     TargetController::{:?}, which no `EnchantControllerConstraint` maps to",
                    other
                ),
            };
            let lowering_says_legal = effects::matches_filter(&chars, &tf) && controller_ok;

            assert_eq!(
                lowering_says_legal, offer_says_legal,
                "PB-DX20b t9 surface 1 vs 2: `matches_filter` against the query's own \
                 TargetFilter disagrees with `legal_targets_per_slot`. [{}] x [{}]: \
                 lowering={} offer={}; tf={:?}",
                fc.label, shape.name, lowering_says_legal, offer_says_legal, tf
            );

            // ── Surface 3: the cast.
            let cast_result =
                process_command(state, cast(p1, aura, vec![Target::Object(candidate)]));
            let cast_says_legal = cast_result.is_ok();
            assert_eq!(
                cast_says_legal,
                offer_says_legal,
                "SR-38 / PB-DX20b t9 surface 3: the offer and the cast disagree — a clean \
                 offer followed by a guaranteed refusal (or a target the picker never showed). \
                 [{}] x [{}]: offer={} cast={} ({:?})",
                fc.label,
                shape.name,
                offer_says_legal,
                cast_says_legal,
                cast_result.err()
            );

            // ── Surface 4: CR 704.5m, the SBA, on the same pair already attached.
            let (attached, attached_aura) = t9_attached_board(&fc.filter, shape);
            let (fell, _) = aura_fell_off(attached, attached_aura);
            assert_eq!(
                !fell, cast_says_legal,
                "PB-DX20b t9 surface 4: CR 303.4a (cast) and CR 704.5m (SBA) disagree about \
                 the SAME (filter, permanent) pair. This is exactly the drift PB-DX20 left \
                 open by keeping two hand-written copies of the field arithmetic. [{}] x [{}]: \
                 cast_legal={} fell_off={}",
                fc.label, shape.name, cast_says_legal, fell
            );

            if cast_says_legal {
                legal_cells += 1;
            }
        }
    }

    // ── Non-vacuity floors. Four surfaces that all say "no" agree perfectly and measure
    //    nothing; so do four that all say "yes".
    assert_eq!(
        cells,
        filters.len() * shapes.len(),
        "t9 must visit every matrix cell"
    );
    assert!(
        cells >= 100,
        "t9's matrix is a floor, not a decoration: {} cells is too few to exercise the \
         has_card_types OR against the has_card_type AND",
        cells
    );
    assert!(
        legal_cells > 0 && legal_cells < cells,
        "t9 non-vacuity: {} of {} cells were legal. All-legal or all-illegal means the matrix \
         agrees trivially and discriminates nothing.",
        legal_cells,
        cells
    );
}
