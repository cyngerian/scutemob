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
//! * `t10` — the per-FIELD MAPPING matrix (`/review` finding 2). Each `EnchantFilter` field
//!   must land in its OWN `TargetFilter` slot and in no other. `r5` decides by field NAME and
//!   `t9` compares two consumers of the SAME lowering, so a field wired into the wrong slot is
//!   invisible to both — see `t10`'s own docstring for the three planted classes and which gate
//!   catches which.
//! * `t11` — CR 704.5m as a **transition** (`/review` finding 5). `t6`/`t6b` build the
//!   attachment already illegal (or already legal) and run the FIRST sweep; `t11` attaches
//!   legally, sweeps and asserts survival, then changes the host's type at Layer 4 and sweeps
//!   again.
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
//! 3. **`t10` and `t11` were each ISOLATED by a plant, not merely shown to redden.** Under the
//!    swap `basic: f.nonbasic, nonbasic: f.basic` in `casting::enchant_filter_to_target_filter`,
//!    `t10` is the **only** red row in this file (10 green) and the core roster's 7 rows and the
//!    simulator channel's 4 are green too — which is the reviewer's defeat reproduced here
//!    before it was fixed. Under a plant that makes the CR 704.5m SBA read the host's **base**
//!    `Characteristics` instead of `expect_characteristics` — the raw-characteristics defect
//!    `legal_actions.rs:1276` really has, described in the channel suite's `c4` docstring —
//!    `t11` is the **only** red row (11 green, `t6` and `t6b` among them). Neither row rides on
//!    another's coverage.
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
    Step, SubType, SuperType, Target, TargetController, TargetFilter, TargetRequirement, ZoneId,
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

// ─────────────────────────────────────────────────────────────────────────────
// t10 — the per-FIELD MAPPING matrix
// ─────────────────────────────────────────────────────────────────────────────

/// One `EnchantFilter` field under test: the filter that moves ONLY that field, and the one
/// `TargetFilter` slot the lowering must move in response.
struct MapRow {
    /// The field's name. It is the same name in both structs — the lowering is
    /// name-preserving by design, and that is precisely what makes `r5`'s name-keyed check
    /// blind to a mis-wiring.
    field: &'static str,
    /// An `EnchantFilter` with ONLY this field off its default.
    filter: EnchantFilter,
    /// Applied to the baseline lowering to build the expected `TargetFilter`.
    expect: fn(&mut TargetFilter),
}

/// The seven `EnchantFilter` slots, compared one at a time and by NAME, so the failure message
/// can say which slot moved instead of printing two structs and leaving the diff to the reader.
///
/// Written out rather than derived: a derived comparison would be `tf == base`, which is the
/// assertion this helper exists to *decompose*.
fn responsible_slots_that_moved(base: &TargetFilter, tf: &TargetFilter) -> Vec<&'static str> {
    let mut out = Vec::new();
    if tf.has_card_type != base.has_card_type {
        out.push("has_card_type");
    }
    if tf.has_card_types != base.has_card_types {
        out.push("has_card_types");
    }
    if tf.has_subtype != base.has_subtype {
        out.push("has_subtype");
    }
    if tf.has_subtypes != base.has_subtypes {
        out.push("has_subtypes");
    }
    if tf.basic != base.basic {
        out.push("basic");
    }
    if tf.nonbasic != base.nonbasic {
        out.push("nonbasic");
    }
    if tf.controller != base.controller {
        out.push("controller");
    }
    out
}

/// The same decomposition on the INPUT side — a fixture guard. A row that sets two
/// `EnchantFilter` fields is no longer a per-field row, and would make `t10` report a
/// two-slot move as a mapping bug.
fn enchant_slots_that_moved(f: &EnchantFilter) -> Vec<&'static str> {
    let base = EnchantFilter::default();
    let mut out = Vec::new();
    if f.has_card_type != base.has_card_type {
        out.push("has_card_type");
    }
    if f.has_card_types != base.has_card_types {
        out.push("has_card_types");
    }
    if f.has_subtype != base.has_subtype {
        out.push("has_subtype");
    }
    if f.has_subtypes != base.has_subtypes {
        out.push("has_subtypes");
    }
    if f.basic != base.basic {
        out.push("basic");
    }
    if f.nonbasic != base.nonbasic {
        out.push("nonbasic");
    }
    if f.controller != base.controller {
        out.push("controller");
    }
    out
}

/// The `TargetFilter` the engine's own lowering produces for `f`, read back through the public
/// query surface.
///
/// `casting::enchant_filter_to_target_filter` is `pub(crate)` and this file does not widen a
/// visibility for a test's convenience (module doc). `queries::spell_target_requirements`
/// returns the lowering's output verbatim — `enchant_target_to_requirement`'s `Filtered` arm is
/// literally `TargetPermanentWithFilter(enchant_filter_to_target_filter(f))` — so this reads the
/// real mapping, not a re-implementation of it.
fn lowered_target_filter(f: &EnchantFilter) -> TargetFilter {
    let p1 = p(1);
    let p2 = p(2);
    // No candidate permanent: `t10`'s subject is the MAPPING, not whether anything matches it.
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(synthetic_aura(p1, f, ZoneId::Hand(p1)))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("t10 board builds");
    let aura = find_object(&state, "T9 Aura");
    let reqs = spell_target_requirements(&state, aura, &[], None, false);
    assert_eq!(
        reqs.len(),
        1,
        "CR 303.4a: an Aura spell announces exactly one target; got {:?}",
        reqs
    );
    match &reqs[0] {
        TargetRequirement::TargetPermanentWithFilter(tf) => tf.clone(),
        other => panic!(
            "PB-DX20b t10: `EnchantTarget::Filtered` must lower to TargetPermanentWithFilter — \
             that IS the mapping under test; got {:?}",
            other
        ),
    }
}

fn map_rows() -> Vec<MapRow> {
    vec![
        MapRow {
            field: "has_card_type",
            filter: EnchantFilter {
                has_card_type: Some(CardType::Enchantment),
                ..Default::default()
            },
            expect: |tf| tf.has_card_type = Some(CardType::Enchantment),
        },
        MapRow {
            field: "has_card_types",
            filter: EnchantFilter {
                has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                ..Default::default()
            },
            expect: |tf| tf.has_card_types = vec![CardType::Creature, CardType::Planeswalker],
        },
        MapRow {
            field: "has_subtype",
            filter: EnchantFilter {
                has_subtype: Some(SubType("Goblin".to_string())),
                ..Default::default()
            },
            expect: |tf| tf.has_subtype = Some(SubType("Goblin".to_string())),
        },
        MapRow {
            field: "has_subtypes",
            filter: EnchantFilter {
                has_subtypes: vec![SubType("Forest".to_string()), SubType("Plains".to_string())],
                ..Default::default()
            },
            expect: |tf| {
                tf.has_subtypes = vec![SubType("Forest".to_string()), SubType("Plains".to_string())]
            },
        },
        MapRow {
            field: "basic",
            filter: EnchantFilter {
                basic: true,
                ..Default::default()
            },
            expect: |tf| tf.basic = true,
        },
        MapRow {
            field: "nonbasic",
            filter: EnchantFilter {
                nonbasic: true,
                ..Default::default()
            },
            expect: |tf| tf.nonbasic = true,
        },
        MapRow {
            field: "controller",
            filter: EnchantFilter {
                controller: EnchantControllerConstraint::You,
                ..Default::default()
            },
            expect: |tf| tf.controller = TargetController::You,
        },
    ]
}

/// The seven fields `EnchantFilter` carries at HEAD, as `t10`'s row set must cover them.
///
/// This is a SECOND pin of the same list `r5` pins in
/// `crates/engine/tests/core/pb_dx20b_enchant_line_roster.rs`, and the duplication is
/// deliberate rather than lazy: the two live in different test targets and cannot share a
/// constant, and `r5` is the row that keys the list to the struct's own declaration. If an
/// eighth field is added, `r5` reddens on the declaration and this floor reddens on the
/// coverage — an eighth field that is lowered but never per-field mapped would otherwise slip
/// past `t10` in silence.
const T10_FIELDS: &[&str] = &[
    "basic",
    "controller",
    "has_card_type",
    "has_card_types",
    "has_subtype",
    "has_subtypes",
    "nonbasic",
];

// ─────────────────────────────────────────────────────────────────────────────
// t12 — `T10_FIELDS` is pinned to `EnchantFilter`'s own declaration (`OOS-DX28-1`)
// ─────────────────────────────────────────────────────────────────────────────

/// Every `pub` field name declared by `pub struct <struct_name>` in a workspace-relative
/// source file.
///
/// # This is a COPY, and the canonical version is named
///
/// The canonical implementation is
/// `crates/engine/tests/core/pb_dx57_declared_source.rs::declared_struct_fields`, which is
/// the ONE declaration parser the field-set fingerprints in the `core` test target are pinned
/// against. It cannot be shared with this file: `crates/engine/tests/*/main.rs` compiles one
/// binary per GROUP, `core` and `primitives` are different binaries, and
/// `tests/no_stray_test_binaries.rs::group_main_rs_declares_modules_and_nothing_else` allows
/// a group's `main.rs` to contain bare `mod x;` lines and nothing else — so a `#[path]`
/// re-export is not available either. The tree's established answer to exactly this situation
/// is `primitives/pb_dp9_effect_choice.rs:2641`: keep the copy, say it is a copy, name the
/// canonical version, and cross-check BY VALUE rather than by text. `t12` does that
/// cross-check (see its doc).
///
/// # Bounds, stated rather than left to be discovered
///
/// This copy strips `//` line comments only, and then ASSERTS that the file carries no
/// `/* */` block comment — PB-DX8's `OOS-DX32-6` defeat (the byte-identical sentence reddened
/// as a line comment and left every test green as a block comment) applies here as much as
/// anywhere, and an assertion is cheaper than a second stripper. It panics on an empty parse,
/// because a parser that returns `{}` makes every `assert_eq!` against it trivially true —
/// which is `OOS-DX28-1`'s own failure mode re-entering through its fix.
fn declared_struct_fields(rel: &str, struct_name: &str) -> std::collections::BTreeSet<String> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("engine manifest dir is <workspace>/crates/engine")
        .to_path_buf();
    let path = root.join(rel);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} must be readable: {e}", path.display()));
    assert!(
        !raw.contains("/*"),
        "{} grew a `/* */` block comment. This parser strips `//` only, so a block comment can \
         hide or fake a field declaration (`OOS-DX32-6`). Widen the stripper, or use the \
         canonical `core::pb_dx57_declared_source::declared_struct_fields`, which handles \
         both.",
        path.display()
    );
    // Length-preserving line-comment strip, so a doc comment mentioning the header cannot
    // mis-anchor the search below.
    let clean: String = raw
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => format!("{}{}", &l[..i], " ".repeat(l.len() - i)),
            None => l.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n");

    let header = format!("pub struct {struct_name} {{");
    let at = clean.find(&header).unwrap_or_else(|| {
        panic!(
            "`{header}` not found in {}. The declaration was renamed, moved, or its visibility \
             changed. Re-point this pin — do NOT delete it and keep the hand-written \
             T10_FIELDS, which is the defect `OOS-DX28-1` names.",
            path.display()
        )
    });
    let body_start = clean[at..].find('{').expect("declaration has a body") + at + 1;
    let mut depth = 1usize;
    let mut end = None;
    for (i, ch) in clean[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(body_start + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end.expect("the struct body is never closed \u{2014} the brace walk ran off the end");
    // Split the body on TOP-LEVEL commas rather than on lines. A line-based split takes only
    // the FIRST `pub X:` per line, so two fields written on one line
    // (`pub basic: bool, pub nonbasic: bool,` -- legal Rust that `cargo fmt` normally
    // splits, and therefore a shape a formatted tree hides) makes the parse SHORT and the
    // failure message say *"the declaration no longer has `nonbasic`"* when the declaration
    // still has it. Found by a live, unrelated plant during PB-DX57 rather than by reasoning:
    // a WRONG diagnosis from a red gate is only one step better than a green one.
    let mut fields: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    let mut prev = ' ';
    let push = |chunk: &str, out: &mut std::collections::BTreeSet<String>| {
        let mut s = chunk.trim();
        // Drop any leading `#[...]` attributes, possibly several.
        while s.starts_with("#[") {
            let mut d = 0usize;
            let mut e = None;
            for (i, ch) in s.char_indices() {
                match ch {
                    '[' => d += 1,
                    ']' => {
                        d -= 1;
                        if d == 0 {
                            e = Some(i + 1);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            match e {
                Some(i) => s = s[i..].trim_start(),
                None => break,
            }
        }
        let Some(rest) = s.strip_prefix("pub ") else {
            return;
        };
        // PB-DX57 adversarial pass: `pub r#type: bool` is a legal field declaration (a field
        // named after a keyword MUST be written that way), and an identifier scan that takes
        // only `[A-Za-z0-9_]` reads it as the EMPTY string and drops the field in silence. That
        // defeated this pin completely — the whole test target stayed green with the field
        // present. Handle the `r#` prefix, and FAIL CLOSED on a `pub ` chunk that still yields
        // nothing: a dropped field is invisible to every consumer at once, which is why the
        // by-value cross-check could not see it either.
        let rest = rest.trim_start();
        let (raw_prefix, rest) = match rest.strip_prefix("r#") {
            Some(r) => ("r#", r),
            None => ("", rest),
        };
        let body: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        assert!(
            !body.is_empty() && rest[body.len()..].trim_start().starts_with(':'),
            "could not parse a `pub` field declaration from {rest:?} -- refusing to return a \
             field set that silently omits it (PB-DX57 adversarial pass)"
        );
        out.insert(format!("{raw_prefix}{body}"));
    };
    for ch in clean[body_start..end].chars() {
        match ch {
            '{' | '(' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '}' | ')' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            '<' if prev.is_ascii_alphanumeric() || prev == '_' => {
                depth += 1;
                cur.push(ch);
            }
            '>' if depth > 0 && prev != '-' && prev != '=' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                let chunk = std::mem::take(&mut cur);
                push(&chunk, &mut fields);
            }
            _ => cur.push(ch),
        }
        prev = ch;
    }
    push(&cur, &mut fields);
    let out = fields;
    assert!(
        !out.is_empty(),
        "parsed ZERO fields out of `{header}` in {} — every assert_eq! against this set would \
         be trivially satisfiable",
        path.display()
    );
    out
}

#[test]
/// `OOS-DX28-1` — **`T10_FIELDS` is a hand-maintained field-set fingerprint, and until this
/// row nothing in this test target compared it to `EnchantFilter`'s own declaration.**
///
/// The seed is the class: `TARGET_FILTER_FIELDS` recognised a serialized node by comparing
/// its key set to a 32-entry `&[&str]`; adding the 33rd field stopped it matching **anything**,
/// with no compile error and a failure message that pointed nowhere near the cause.
///
/// ## What `T10_FIELDS`'s own doc got half-right, and why that half matters
///
/// It says the duplication is safe because *"if an eighth field is added, `r5` reddens on the
/// declaration and this floor reddens on the coverage"*. The first clause is true —
/// `core::pb_dx20b_enchant_line_roster::r5` does key its list to the declaration. The second
/// is **not**, and it is the load-bearing one: `t10`'s `covered` and `expected` are BOTH
/// hand-maintained (`covered` comes from `map_rows()`, `expected` from `T10_FIELDS`), so an
/// eighth field that is lowered but given no `MapRow` leaves both sides unchanged and `t10`
/// passes while its per-field mapping matrix is one row short. Executed: planting an eighth
/// `EnchantFilter` field reddens `r5` (a different test binary) and **not** `t10` — and `t10`
/// is the only row in the tree that catches a mis-WIRING, which is a class `r5` is
/// structurally blind to (it decides by field NAME, and a swap reads both names).
///
/// So the pin is DUPLICATED here rather than delegated. Two test binaries cannot share a
/// constant; they can each hold the same assertion.
///
/// ## The by-value cross-check
///
/// `declared_struct_fields` above is a copy of
/// `core::pb_dx57_declared_source::declared_struct_fields`. Two parsers that agree only with
/// themselves are worth nothing, so this row asserts the same VALUE the canonical side
/// asserts: `core::pb_dx57_declared_source::p1_the_parser_agrees_with_the_independent_parsers_already_in_the_tree`
/// contains `assert_eq!(declared_struct_fields(STATE_TYPES_RS, "EnchantFilter").len(), 7)`.
/// If the two parsers ever disagree about this struct, one of the two numbers moves and the
/// other does not.
fn t12_t10_fields_is_pinned_to_the_enchant_filter_declaration() {
    let declared = declared_struct_fields("crates/card-types/src/state/types.rs", "EnchantFilter");
    let pinned: std::collections::BTreeSet<String> =
        T10_FIELDS.iter().map(|s| (*s).to_string()).collect();

    assert_eq!(
        declared,
        pinned,
        "`OOS-DX28-1`: T10_FIELDS no longer matches `pub struct EnchantFilter`'s declaration \
         in crates/card-types/src/state/types.rs. declared only: {:?}; pinned only: {:?}. A new \
         field needs BOTH a `T10_FIELDS` entry AND a `MapRow` in `map_rows()` — `t10`'s two \
         sides are both hand-maintained, so adding neither leaves `t10` green while its mapping \
         matrix is short.",
        declared.difference(&pinned).collect::<Vec<_>>(),
        pinned.difference(&declared).collect::<Vec<_>>()
    );

    // By-value cross-check against the canonical parser in the `core` binary. See the doc.
    assert_eq!(
        declared.len(),
        7,
        "the local declaration parser reads {} EnchantFilter fields; \
         core::pb_dx57_declared_source::p1 asserts the canonical parser reads 7. If the struct \
         really grew, BOTH numbers move in the same commit; if only one moved, one of the two \
         parsers is wrong and reconciling by editing whichever is easier to change is how the \
         seed's class returns.",
        declared.len()
    );
}

#[test]
/// CR 702.5a — **the per-field MAPPING matrix.** Each `EnchantFilter` field must land in its
/// OWN `TargetFilter` slot, and in no other.
///
/// ## Why this is not `r5`'s job and not `t9`'s — both were defeated by the same plant
///
/// `/review` finding 2 proved this by execution, and the plant is two swapped lines in
/// `casting::enchant_filter_to_target_filter`:
///
/// ```text
///         basic: f.nonbasic,
///         nonbasic: f.basic,
/// ```
///
/// With that in the tree, **all 23 of PB-DX20b's own new tests stayed GREEN** — while
/// `ossification` and `dimensional_exile` (both `Complete`, both deck-legal, both declaring
/// `has_card_type: Land, basic: true, controller: You`) refused every basic land and accepted
/// only nonbasic ones. Two live cards, wrong in the browser, and this batch's whole suite
/// silent.
///
/// The 23 were **re-executed here rather than accepted from the report**, and they reproduce
/// exactly: 10 in this file, 7 in `core::pb_dx20b_enchant_line_roster` (`r5` among them),
/// 4 in `mtg-simulator`'s `pb_dx20b_enchant_offer_channel`, the `play-server` HTTP probe
/// `test_dx20b_imprisoned_offer_excludes_the_artifact_over_http`, and
/// `core::pb_dx49_saga_blanking_roster::r4a_pair_a_is_dead_since_oos_dx20_10_closed`. With
/// `t10` in the tree the swap reddens `t10` and nothing else.
///
/// The reasons are structural and neither gate can be repaired into covering the other:
///
/// * **`r5` decides by field NAME.** It asserts *declared ⊆ lowered*, where "lowered" is the
///   set of `f.<name>` reads in the function body. A swap reads both names, so both sets are
///   unchanged and `r5` is green by construction. `r5` catches a field that is never READ; it
///   is blind to a field that is read into the wrong slot.
/// * **`t9` compares two CONSUMERS of the same lowering.** The offer, the cast and the CR 704.5m
///   SBA all consume `enchant_filter_to_target_filter`'s output — PB-DX20b's own structural
///   change is what made that true — so a wrong lowering makes all three wrong in the same
///   direction and they agree perfectly. `t9`'s own docstring already says it is a consistency
///   pin; this row is the correctness pin for the mapping itself.
///
/// The swap survived only because three tests in
/// `crates/engine/tests/mechanics_e_l/enchant.rs` — a file PB-DX20b never touched — happen to
/// exercise `basic` and `nonbasic` behaviourally. Relying on that is relying on a neighbour.
///
/// ## Why a row per FIELD, when only one PAIR can be swapped — three classes, all executed
///
/// A compile-silent SWAP needs two fields of the same type, and `EnchantFilter`'s only
/// same-typed pair at HEAD is `basic`/`nonbasic` (both `bool`). Every other field has a distinct
/// type, so exchanging two of them would not compile — *today*. On that axis alone one row for
/// the bool pair would do. Two further classes are why there are seven:
///
/// | planted defect | `r5` | `t10` |
/// |---|---|---|
/// | SWAP — `basic: f.nonbasic, nonbasic: f.basic` | **green** | **RED** (`moved ["nonbasic"], want ["basic"]`) |
/// | DROP — `basic: false`, the read deleted | **RED** (`declared but never read: ["basic"]`) | **RED** (`moved [], want ["basic"]`) |
/// | DISCARD — `has_subtypes: f.has_subtypes.iter().take(0).cloned().collect()` | **green** | **RED** (`moved [], want ["has_subtypes"]`) |
///
/// Every cell above was executed, not reasoned. The DROP row is the one `r5` already covers,
/// and it is stated rather than claimed for this row: `r5` keys on the `f.<name>` READ, so
/// deleting the read reddens it. The DISCARD row is what `r5` structurally cannot see — the
/// token appears, the value never arrives — which is `OOS-DX7-2`'s shape one struct over, and it
/// is compile-silent for **all seven** fields regardless of their types. That is why the matrix
/// is per-field rather than one row for the bool pair.
///
/// ## What each row asserts
///
/// (a) the field's own slot carries the mapped value, and (b) **every other slot** is still at
/// the baseline — where the baseline is the lowering of `EnchantFilter::default()`, taken from
/// the engine rather than assumed. (b) is what turns a swap into a red test: under the plant
/// above, the `basic` row moves `nonbasic` and vice versa, so both rows fail on (a) *and* on (b).
///
/// The final `assert_eq!` compares the WHOLE `TargetFilter`, so a lowering that started writing
/// a slot outside the seven — `legendary`, `is_token`, `max_cmc` — also reddens. PB-DX20 §3.4
/// chose `..Default::default()` deliberately; this row is what keeps that choice honest.
fn t10_every_enchant_filter_field_maps_to_its_own_target_filter_slot() {
    let base = lowered_target_filter(&EnchantFilter::default());
    let rows = map_rows();

    // ── Coverage floor: every declared field has a row. See `T10_FIELDS`.
    let covered: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.field).collect();
    let expected: std::collections::BTreeSet<&str> = T10_FIELDS.iter().copied().collect();
    assert_eq!(
        covered,
        expected,
        "PB-DX20b t10: the row set must cover every EnchantFilter field exactly once. \
         rows only: {:?}; pinned only: {:?}",
        covered.difference(&expected).collect::<Vec<_>>(),
        expected.difference(&covered).collect::<Vec<_>>()
    );

    for row in &rows {
        // ── Fixture guard: the row really moves exactly one INPUT field.
        let moved_in = enchant_slots_that_moved(&row.filter);
        assert_eq!(
            moved_in,
            vec![row.field],
            "PB-DX20b t10 fixture: the `{}` row must set exactly that field on its \
             EnchantFilter — a row that moves two inputs cannot attribute a two-slot output \
             move to a mapping bug. Moved: {:?}",
            row.field,
            moved_in
        );

        let tf = lowered_target_filter(&row.filter);

        // ── (b), by NAME, so the message says which slot went where.
        let moved_out = responsible_slots_that_moved(&base, &tf);
        assert_eq!(
            moved_out,
            vec![row.field],
            "CR 702.5a / `/review` finding 2: `EnchantFilter.{}` must land in \
             `TargetFilter.{}` and in NO other slot. The lowering moved {:?} instead. This is \
             the class `r5` cannot see (it decides by field NAME, and a mis-wiring reads both \
             names) and `t9` cannot see (it compares two consumers of THIS lowering, so a wrong \
             mapping makes them agree). baseline={:?} got={:?}",
            row.field,
            row.field,
            moved_out,
            base,
            tf
        );

        // ── (a) + everything outside the seven slots, in one total comparison.
        let mut want = base.clone();
        (row.expect)(&mut want);
        assert_eq!(
            tf, want,
            "CR 702.5a: `EnchantFilter.{}` did not arrive in `TargetFilter.{}` with its own \
             value. A field whose read appears in the function body but whose VALUE never \
             reaches the TargetFilter is `OOS-DX7-2`'s shape, and `r5` — which keys on the \
             read, not on the value — is green for it.",
            row.field, row.field
        );

        // ── Non-vacuity: a row that changed nothing would satisfy nothing above except by
        //    making `moved_out` empty, which the assertion catches — but say it directly, so a
        //    future edit that makes the baseline and the row identical fails HERE with a
        //    message about the fixture.
        assert_ne!(
            tf, base,
            "PB-DX20b t10 non-vacuity: the `{}` row lowered to the baseline TargetFilter, so it \
             measures nothing. Its distinguishing value is no longer distinguishing.",
            row.field
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// t11 — CR 704.5m as a TRANSITION: legally attached, then made illegal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
/// CR 704.5m / 613.1d — an Aura that is **legally** attached and then *becomes* illegally
/// attached is put into its owner's graveyard at the next state-based check.
///
/// ## Why this row exists — `t6`/`t6b` do not cover it, and AC 7309 names it
///
/// `t6` and `t6b` build the board with the Aura **already** attached to a printed-illegal (or
/// printed-legal) permanent and run the FIRST state-based sweep. That is a from-the-start
/// illegality: no sweep ever observed the attachment as legal. The case CR 704.5m actually
/// fires in during a real game is the other one — the Aura is attached legally (by resolving,
/// by an effect that puts it onto the battlefield attached, by a control change) and the board
/// then moves under it.
///
/// The two are different because a SBA that only ever sees the illegal state cannot
/// distinguish "this was never legal" from "this stopped being legal", and an implementation
/// that latched the legality at attach time — cached it on the `GameObject`, or checked it only
/// on the attach event — would pass `t6`, pass `t6b`, and leave the Aura on the board forever.
/// This row is the only thing in the file that separates those two implementations.
///
/// ## How the transition is produced, and why it is a real one
///
/// The precondition is **executed, not assumed**: the first sweep runs with the Aura on a
/// vanilla 2/2 creature and is asserted NOT to detach it, and the host's layer-resolved card
/// types are asserted to contain `Creature` at that instant. Only then is the board changed.
///
/// The change is a **Layer 4 type-changing continuous effect** (CR 613.1d) —
/// `EffectLayer::TypeChange` + `LayerModification::SetCardTypes({Artifact})` filtered to the
/// host alone — so the host stops being a creature the way the rules say a permanent stops
/// being a creature, and `sba::matches_enchant_target` has to reach it through
/// `calculate_characteristics`. It is **not** produced by editing the host's base
/// `Characteristics` (which would make the layer axis untested) and **not** by moving the Aura
/// (which is CR 704.5m's other antecedent and a different claim).
///
/// **Disclosed rather than glossed**: the continuous effect is installed by pushing onto
/// `state.continuous_effects_mut()` rather than by resolving a card that grants it. No card in
/// this fixture makes another permanent an artifact, and `GameStateBuilder::build()` registers
/// no statics at all (`OOS-DX43-6`), so there is no card-driven route on this board. What that
/// costs is stated precisely: the *installation* is synthetic, the *evaluation* is not — the
/// effect goes through the same layer walk every real effect does, and it is installed AFTER a
/// state-based sweep has already run and passed, which is the property this row is about. The
/// same construct is how `crates/engine/tests/mechanics_e_l/enchant.rs` builds its layer
/// fixtures (`test_animate_land_pt_and_types_via_chained_or_awaken`).
///
/// CR 400.7: the detached Aura is a NEW object in the graveyard, so the battlefield `ObjectId`
/// is dead there and the destination is asserted by NAME and ZONE — the same discipline `t6`
/// uses, and the reason `aura_fell_off`'s event check and the zone check are two separate
/// assertions rather than one lookup.
fn t11_a_legally_attached_aura_falls_off_when_the_host_changes_type() {
    use mtg_engine::{
        check_and_apply_sbas, ContinuousEffect, EffectDuration, EffectFilter, EffectId,
        EffectLayer, LayerModification,
    };

    // ── Phase 1: attach legally, sweep, and prove the sweep left it alone.
    let (state, aura) = attached_board(ObjectSpec::creature(p(1), "Attach Victim", 2, 2));
    let (fell_first, mut after) = aura_fell_off(state, aura);
    assert!(
        !fell_first,
        "PRECONDITION: Imprisoned in the Moon on a creature is LEGALLY attached (\"Enchant \
         creature, land, or planeswalker\") and the first state-based sweep must leave it \
         alone. If this fires the probe never had a legal state to transition out of, and \
         everything below would measure a from-the-start illegality — which is `t6`'s subject, \
         not this row's."
    );

    let victim = find_object(&after, "Attach Victim");
    let before_types = mtg_engine::calculate_characteristics(&after, victim)
        .expect("the host is on the battlefield")
        .card_types
        .clone();
    assert!(
        before_types.contains(&CardType::Creature),
        "PRECONDITION: the host must be a creature at the instant the first sweep passed, or \
         the sweep passed for some other reason. Layer-resolved types: {:?}",
        before_types
    );
    assert_eq!(
        after
            .objects()
            .get(&aura)
            .and_then(|a| a.attached_to)
            .as_ref(),
        Some(&victim),
        "PRECONDITION: the Aura must still be attached to the host after the first sweep"
    );

    // ── Phase 2: the transition. CR 613.1d, Layer 4 — the host becomes an artifact and stops
    //    being a creature. Nothing about the Aura moves.
    after.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(70_451),
        source: None,
        layer: EffectLayer::TypeChange,
        modification: LayerModification::SetCardTypes([CardType::Artifact].into_iter().collect()),
        filter: EffectFilter::SingleObject(victim),
        duration: EffectDuration::Indefinite,
        condition: None,
        is_cda: false,
        affected_set: None,
        timestamp: 900_000,
    });

    let now_types = mtg_engine::calculate_characteristics(&after, victim)
        .expect("the host is still on the battlefield")
        .card_types
        .clone();
    assert!(
        !now_types.contains(&CardType::Creature) && now_types.contains(&CardType::Artifact),
        "PRECONDITION: the Layer 4 effect must actually have changed the host's LAYER-RESOLVED \
         types — if it had not, the SBA below would be asked the same question as phase 1 and \
         this row would silently become a duplicate of `t6b`. Types: {:?}",
        now_types
    );

    // ── Phase 3: the next state-based check. CR 704.5m.
    let events = check_and_apply_sbas(&mut after);
    let fired = events
        .iter()
        .any(|e| matches!(e, GameEvent::AuraFellOff { object_id, .. } if *object_id == aura));
    assert!(
        fired,
        "CR 704.5m: the host is now an artifact and nothing else, which \"Enchant creature, \
         land, or planeswalker\" does not admit, so the Aura is illegally attached and its \
         controller puts it into its owner's graveyard. It was LEGALLY attached one sweep ago \
         — a legality latched at attach time passes `t6` and `t6b` and fails only here. \
         Events: {:?}",
        events
    );

    // CR 400.7 — the detached Aura is a NEW object; assert the destination by name and zone.
    let in_graveyard = after.objects().iter().any(|(_, o)| {
        o.characteristics.name == "Imprisoned in the Moon" && o.zone == ZoneId::Graveyard(p(1))
    });
    assert!(
        in_graveyard,
        "CR 704.5m: the detached Aura goes to its OWNER's graveyard. `AuraFellOff` alone says \
         it came off, never where it landed. Objects at rest: {:?}",
        after
            .objects()
            .iter()
            .map(|(_, o)| (o.characteristics.name.clone(), o.zone))
            .collect::<Vec<_>>()
    );
    assert!(
        !after
            .objects()
            .iter()
            .any(|(_, o)| o.characteristics.name == "Imprisoned in the Moon"
                && o.zone == ZoneId::Battlefield),
        "CR 704.5m: no copy of the Aura may remain on the battlefield after it falls off"
    );
}
