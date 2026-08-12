//! PB-DX26 (`OOS-CARDS1-3` + `OOS-CARDS1-1` + `OOS-DX3b-1`): the attach surface,
//! one link earlier than CARDS-1.
//!
//! **The defect.** `state/keyword_registry.rs:98` classifies `K::Equip` as a
//! `KeywordHandling::Marker` whose carrier is "`Effect::AttachEquipment` …
//! activated through `AbilityDefinition::Activated`". A marker synthesises
//! **nothing**. So 21 defs carrying only
//! `AbilityDefinition::Keyword(KeywordAbility::Equip)` had:
//!
//!   * no `ActivatedAbility` in their layer-resolved characteristics, therefore
//!   * nothing for `StubProvider`/`legal_actions.rs` to offer, therefore
//!   * no ability index a `Command::ActivateAbility` could name.
//!
//! Where `OOS-M11-10(equip)` was *"the picker never asks for a target"*, this is
//! ***"there is no action to pick"*** — the same playtest symptom, one link
//! sooner, and on a strictly larger population (21 defs, **10 of them deck-legal
//! `Complete`**, versus CARDS-1's 17).
//!
//! **Fail-before evidence is executed, not argued.** T1/T2/T5/T7/T9 all fail
//! against the pre-fix corpus; the verbatim pre- and post-fix output is in
//! `memory/primitives/pb-DX26-fail-before-2026-08-11.md`. The corresponding
//! roster-level measurement (`R2 = 21 of 21` marker-without-ability) is in
//! `core::pb_dx26_attach_keyword_roster`.
//!
//! **T3 is the index hazard, stated rather than assumed.** Authoring a new
//! `AbilityDefinition::Activated` into a def that already has one MOVES the
//! activated-ability indices a `Command::ActivateAbility` names. Umezawa's Jitte is
//! the only member of the 21 with a pre-existing activated ability (the PB-EF7
//! modal counter-removal), so it is the only def where that could bite; T3 pins
//! both abilities' order explicitly so the ordering is a checked fact rather than
//! an accident of where an editor happened to paste.

use mtg_engine::state::GameStateError;
use mtg_engine::{
    ability_target_requirements, all_cards, calculate_characteristics, card_name_to_id,
    enrich_spec_from_def, legal_targets_per_slot, process_command, CardDefinition, CardRegistry,
    Command, Effect, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaColor, ObjectId,
    ObjectSpec, PlayerId, Step, SubType, Target, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn find_object_controlled_by(state: &GameState, name: &str, controller: PlayerId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.controller == controller)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' controlled by {controller:?} not found"))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

fn activate(
    state: GameState,
    player: PlayerId,
    source: ObjectId,
    ability_index: usize,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index,
            targets,
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
}

/// Two players, `card_name` on p1's battlefield (enriched from the real def and
/// with its static continuous effects registered — `GameStateBuilder::object()`
/// bypasses the ETB machinery that normally does this, exactly as
/// `cards1_equip_target_repair.rs`'s setup documents), one creature each, p1 has
/// priority in its own precombat main with `mana` colourless in pool.
fn setup(
    card_name: &str,
    mana: u32,
) -> (GameState, ObjectId, ObjectId, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let subject = enrich_spec_from_def(
        ObjectSpec::card(p1, card_name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id(card_name)),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(subject)
        .object(ObjectSpec::creature(p1, "P1 Bear", 2, 2))
        .object(ObjectSpec::creature(p2, "P2 Bear", 2, 2))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, mana);
    state.turn_mut().priority_holder = Some(p1);

    let subject_id = find_object(&state, card_name);
    let p1_creature_id = find_object_controlled_by(&state, "P1 Bear", p1);
    let p2_creature_id = find_object_controlled_by(&state, "P2 Bear", p2);

    let card_id = state
        .objects()
        .get(&subject_id)
        .and_then(|o| o.card_id.clone());
    let registry = state.card_registry().clone();
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        subject_id,
        card_id.as_ref(),
        &registry,
        false,
    );

    (state, subject_id, p1_creature_id, p2_creature_id, p1, p2)
}

fn chars_of(state: &GameState, id: ObjectId) -> mtg_engine::Characteristics {
    calculate_characteristics(state, id)
        .unwrap_or_else(|| panic!("object {id:?} must have layer-resolved characteristics"))
}

/// Layer-resolved activated abilities of `id`, in the order a
/// `Command::ActivateAbility { ability_index }` indexes them.
fn activated_descriptions(state: &GameState, id: ObjectId) -> Vec<String> {
    chars_of(state, id)
        .activated_abilities
        .iter()
        .map(|a| a.description.clone())
        .collect()
}

/// Index of the (single) activated ability whose effect is an attach, or `None`.
fn equip_ability_index(state: &GameState, id: ObjectId) -> Option<usize> {
    chars_of(state, id)
        .activated_abilities
        .iter()
        .position(|a| matches!(a.effect, Some(Effect::AttachEquipment { .. })))
}

/// The Fortify sibling of [`equip_ability_index`].
///
/// **PB-DX26 fix cycle (review Finding L10).** `t7`/`t8` hardcoded `ability_index:
/// 0`, which is the very assumption `OOS-DX26-3` was filed about: the index is
/// positional over declaration order, so a later batch authoring another activated
/// ability into `darksteel_garrison` would silently retarget these probes at the
/// wrong ability rather than failing. Locate it the same way equip does.
fn fortify_ability_index(state: &GameState, id: ObjectId) -> Option<usize> {
    chars_of(state, id)
        .activated_abilities
        .iter()
        .position(|a| matches!(a.effect, Some(Effect::AttachFortification { .. })))
}

/// The candidate list for each declared target slot of `source`'s ability
/// `ability_index`, as the browser picker computes it: requirements from
/// `queries::ability_target_requirements`, candidates from
/// `queries::legal_targets_per_slot`. Never re-derives legality here — that is the
/// whole point of asking the engine query rather than a second arithmetic.
fn slot_candidates(
    state: &GameState,
    source: ObjectId,
    ability_index: usize,
    caster: PlayerId,
) -> Vec<Vec<Target>> {
    let reqs = ability_target_requirements(state, source, ability_index);
    legal_targets_per_slot(state, caster, source, &reqs)
}

// ── T1: the offer half — the ability EXISTS and names its target slot ──────────

/// **The core defect, stated as the engine sees it.** Bone Saw is `Complete` and
/// deck-legal; before PB-DX26 its layer-resolved characteristics carried **zero**
/// activated abilities, so `equip_ability_index` returned `None` and no client
/// could construct a `Command::ActivateAbility` naming it at all. Post-fix there is
/// exactly one, it declares exactly one target slot (CR 702.6a), and the candidate
/// list for that slot is **p1's own creature and not p2's** — the "you control"
/// clause, checked through the same `legal_targets_per_slot` query the browser
/// picker reads rather than by re-deriving it here.
#[test]
fn t1_bone_saw_equip_ability_exists_and_offers_exactly_its_own_controllers_creature() {
    let (state, saw_id, p1_creature_id, p2_creature_id, p1, _p2) = setup("Bone Saw", 1);

    let idx = equip_ability_index(&state, saw_id).unwrap_or_else(|| {
        panic!(
            "PB-DX26 / OOS-CARDS1-3: Bone Saw has NO activated equip ability. \
             `keyword_registry.rs`'s `K::Equip` arm is a `KeywordHandling::Marker` — it \
             synthesises nothing — so a def carrying only \
             `AbilityDefinition::Keyword(KeywordAbility::Equip)` has no ability for the \
             provider to offer and no index a `Command::ActivateAbility` could name. \
             There is no action to pick. Layer-resolved activated abilities: {:?}",
            activated_descriptions(&state, saw_id)
        )
    });

    let reqs = ability_target_requirements(&state, saw_id, idx);
    assert_eq!(
        reqs.len(),
        1,
        "CR 702.6a: the equip ability must declare exactly one target slot; found {reqs:?}"
    );

    let candidates = slot_candidates(&state, saw_id, idx, p1);
    assert_eq!(
        candidates.len(),
        1,
        "one requirement means one candidate slot"
    );
    assert!(
        candidates[0].contains(&Target::Object(p1_creature_id)),
        "CR 702.6a 'target creature you control': the activating player's own creature must \
         be offered. Slot: {:?}",
        candidates[0]
    );
    assert!(
        !candidates[0].contains(&Target::Object(p2_creature_id)),
        "CR 702.6a: an OPPONENT's creature must NOT be offered — the requirement carries \
         `controller: TargetController::You`. Slot: {:?}",
        candidates[0]
    );
}

// ── T2: end to end — the activation attaches and the static applies ────────────

/// CR 702.6a/702.6b end-to-end on a real `Complete` def: activating Bone Saw's
/// Equip {1} targeting the controller's own creature pays, resolves, attaches, and
/// the printed "+1/+0" static reaches layer-resolved characteristics (2/2 -> 3/2).
///
/// The P/T half matters beyond "did it attach": Bone Saw's static is filtered by
/// `EffectFilter::AttachedCreature`, so before the equip ability existed the static
/// was live in `continuous_effects` and matched **nothing, forever** — the card was
/// a blank {0} artifact with a name.
#[test]
fn t2_bone_saw_equip_attaches_and_the_printed_static_applies() {
    let (state, saw_id, p1_creature_id, _p2_creature_id, p1, p2) = setup("Bone Saw", 1);
    let idx = equip_ability_index(&state, saw_id).expect("Bone Saw must have an equip ability");

    let before = chars_of(&state, p1_creature_id);
    assert_eq!(
        (before.power, before.toughness),
        (Some(2), Some(2)),
        "sanity: the unequipped bear is 2/2"
    );

    let (state, _) = activate(state, p1, saw_id, idx, vec![Target::Object(p1_creature_id)])
        .expect("activating Bone Saw's equip on the controller's own creature must succeed");
    let (state, events) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state
            .objects()
            .get(&saw_id)
            .expect("saw exists")
            .attached_to,
        Some(p1_creature_id),
        "Bone Saw must be attached to the targeted creature"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::EquipmentAttached { equipment_id, target_id, .. }
            if *equipment_id == saw_id && *target_id == p1_creature_id
        )),
        "an EquipmentAttached event must be emitted; got {events:?}"
    );

    let after = chars_of(&state, p1_creature_id);
    assert_eq!(
        (after.power, after.toughness),
        (Some(3), Some(2)),
        "Bone Saw's printed '+1/+0' (EffectFilter::AttachedCreature, layer 7c) must apply \
         once the equipment is actually attached"
    );
}

// ── T3: the index hazard on the one def that has another activated ability ─────

/// **Authoring an ability moves indices.** Umezawa's Jitte is the only one of
/// PB-DX26's 21 defs with a pre-existing `AbilityDefinition::Activated` (the PB-EF7
/// modal "Remove a charge counter: choose one —"), so it is the only def where
/// appending the equip ability could renumber a `Command::ActivateAbility` that
/// other tests and golden scripts already name.
///
/// This pins the order explicitly rather than trusting it: the modal counter
/// ability stays at index 0 and the new equip ability is appended after it. If a
/// future edit reorders `umezawas_jitte.rs`'s `abilities` vec, this fails here
/// with the reason, instead of somewhere far away with a confusing modal error.
#[test]
fn t3_umezawas_jitte_equip_is_appended_and_does_not_renumber_the_modal_ability() {
    let (state, jitte_id, p1_creature_id, _p2, p1, p2) = setup("Umezawa's Jitte", 2);

    let descriptions = activated_descriptions(&state, jitte_id);
    assert_eq!(
        descriptions.len(),
        2,
        "Umezawa's Jitte must expose exactly two activated abilities: the PB-EF7 modal \
         counter-removal and PB-DX26's Equip {{2}}. Found: {descriptions:?}"
    );

    let equip_idx =
        equip_ability_index(&state, jitte_id).expect("Jitte must have an equip ability post-fix");
    assert_eq!(
        equip_idx, 1,
        "the equip ability must be APPENDED (index 1), leaving the modal counter-removal \
         ability at index 0 where `pb_os10_singleton_cleanup.rs` and any golden script \
         already name it. Found the equip ability at index {equip_idx}; abilities are \
         {descriptions:?}"
    );

    // ...and it really works, not merely exists at the right index.
    let (state, _) = activate(
        state,
        p1,
        jitte_id,
        equip_idx,
        vec![Target::Object(p1_creature_id)],
    )
    .expect("Jitte's Equip {2} must be activatable on the controller's own creature");
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state
            .objects()
            .get(&jitte_id)
            .expect("jitte exists")
            .attached_to,
        Some(p1_creature_id),
        "Umezawa's Jitte must attach — it is `Completeness::Complete` and deck-legal, so a \
         human can put it in a real deck today"
    );
}

// ── T4: the "you control" clause is enforced, not merely offered ────────────────

/// CR 702.6a — an opponent's creature is not just absent from the offer (T1), it is
/// REJECTED if a client names it anyway. Offer-side and validation-side are
/// different code paths (`queries::legal_targets_per_slot` vs
/// `casting::validate_targets_inner`); PB-DX20's durable lesson is that agreement
/// between two consumers of one function proves consistency, not correctness, so
/// both sides are checked.
///
/// **This probe does NOT discriminate the requirement, and that is measured, not
/// assumed.** Weakening `bone_saw`'s requirement to a bare `TargetCreature` leaves
/// this test GREEN (revert matrix row V4b in
/// `memory/primitives/pb-DX26-fail-before-2026-08-11.md`), because `OOS-DX20-7`'s
/// legacy `Effect::AttachEquipment` special-case in `rules/abilities.rs` separately
/// validates a *volunteered* target's creature-ness and controller — it simply never
/// *required* a target, which is why `OOS-M11-10(equip)` was a silent fizzle rather
/// than a visible error. So the rejection here has two independent providers and
/// this assertion cannot tell which one answered.
///
/// The "you control" clause is proven instead by **T1** (offer side, row V4c) and by
/// `core::cards1_equip_target_roster::r2` (shape side, row V4); both go red under
/// the same reversion. T4 is kept as a behavioural pin of the CR rule at the command
/// boundary — it catches a regression that removed *both* providers — and must not
/// be read as evidence about the requirement on its own.
#[test]
fn t4_bone_saw_equip_rejects_an_opponents_creature() {
    let (state, saw_id, _p1_creature_id, p2_creature_id, p1, _p2) = setup("Bone Saw", 1);
    let idx = equip_ability_index(&state, saw_id).expect("Bone Saw must have an equip ability");

    let result = activate(state, p1, saw_id, idx, vec![Target::Object(p2_creature_id)]);
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "CR 702.6a 'target creature YOU control': equipping an opponent's creature must be \
         rejected with InvalidTarget; got {:?}",
        result.map(|(_, ev)| ev)
    );
}

// ── T5: a zero-target activation is refused (the CARDS-1 property, now reachable) ─

/// CR 601.2c — with a mandatory single-target requirement declared, a zero-target
/// activation is rejected by the general check in `rules/abilities.rs`.
///
/// **This probe could not even be written before PB-DX26.** CARDS-1 proved the same
/// property for Skullclamp, but for Bone Saw there was no ability to activate with
/// zero targets: the pre-fix failure is `ability_index: 0` naming nothing, not a
/// permissive target check. The two defects are one link apart and this is the
/// probe that distinguishes them.
#[test]
fn t5_bone_saw_zero_target_activation_is_rejected() {
    let (state, saw_id, _c1, _c2, p1, _p2) = setup("Bone Saw", 1);
    let idx = equip_ability_index(&state, saw_id).expect("Bone Saw must have an equip ability");

    let result = activate(state, p1, saw_id, idx, vec![]);
    match result {
        Err(GameStateError::InvalidTarget(msg)) => assert!(
            msg.contains("expected 1..=1 target(s) but got 0"),
            "expected the target-count-range rejection, got: {msg:?}"
        ),
        other => panic!(
            "a zero-target equip activation must be rejected (CR 601.2c); got {:?}",
            other.map(|(_, ev)| ev)
        ),
    }
}

// ── T6: the keyword marker survives the repair ────────────────────────────────

/// The repair ADDS the activated ability; it does not replace the
/// `AbilityDefinition::Keyword(KeywordAbility::Equip)` marker. The card really does
/// have the keyword — `view-model::format_keyword` renders it and `state/hash.rs`
/// hashes it — so dropping the marker would change what the card *is* in order to
/// fix what it *does*. Pinned so a future "tidy the duplicate" edit fails here.
#[test]
fn t6_equip_keyword_marker_is_retained_alongside_the_authored_ability() {
    let (state, saw_id, _c1, _c2, _p1, _p2) = setup("Bone Saw", 1);
    let chars = chars_of(&state, saw_id);
    assert!(
        chars.keywords.contains(&KeywordAbility::Equip),
        "Bone Saw must still carry the Equip keyword after the ability was authored; \
         keywords: {:?}",
        chars.keywords
    );
    assert!(
        equip_ability_index(&state, saw_id).is_some(),
        "...and the authored ability must be there too — the point is BOTH"
    );
}

// ── T7/T8: Fortify (OOS-CARDS1-1) — the same defect, a DIFFERENT requirement ────

/// Two players, Darksteel Garrison and a land each, p1 with 3 colourless.
fn setup_fortify() -> (
    GameState,
    ObjectId,
    ObjectId,
    ObjectId,
    ObjectId,
    PlayerId,
    PlayerId,
) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let garrison = enrich_spec_from_def(
        ObjectSpec::card(p1, "Darksteel Garrison")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Darksteel Garrison")),
        &defs,
    );

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(garrison)
        .object(ObjectSpec::land(p1, "P1 Wastes"))
        .object(ObjectSpec::land(p2, "P2 Wastes"))
        .object(ObjectSpec::creature(p1, "P1 Bear", 2, 2))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.turn_mut().priority_holder = Some(p1);

    let garrison_id = find_object(&state, "Darksteel Garrison");
    let p1_land_id = find_object_controlled_by(&state, "P1 Wastes", p1);
    let p2_land_id = find_object_controlled_by(&state, "P2 Wastes", p2);
    let p1_creature_id = find_object_controlled_by(&state, "P1 Bear", p1);

    let card_id = state
        .objects()
        .get(&garrison_id)
        .and_then(|o| o.card_id.clone());
    let registry = state.card_registry().clone();
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        garrison_id,
        card_id.as_ref(),
        &registry,
        false,
    );

    (
        state,
        garrison_id,
        p1_land_id,
        p2_land_id,
        p1_creature_id,
        p1,
        p2,
    )
}

/// CR 702.67a — Fortify {3} offers exactly one slot, and that slot contains p1's
/// own **land**: not p2's land (the "you control" half) and not p1's creature (the
/// half that would break if the equip repair's `TargetCreatureWithFilter` had been
/// copied verbatim, which is why `OOS-CARDS1-1` says so in as many words).
#[test]
fn t7_darksteel_garrison_fortify_offers_only_a_land_its_controller_owns() {
    let (state, garrison_id, p1_land_id, p2_land_id, p1_creature_id, p1, _p2) = setup_fortify();

    let fortify_idx = fortify_ability_index(&state, garrison_id)
        .expect("Darksteel Garrison must have an activated Fortify ability");
    let reqs = ability_target_requirements(&state, garrison_id, fortify_idx);
    assert_eq!(
        reqs.len(),
        1,
        "CR 702.67a: Fortify must declare exactly one target slot. Before PB-DX26 this was \
         `targets: vec![]` — zero slots, so the browser picker never asked, the {{3}} was \
         paid, and the attach fizzled in silence. Found: {reqs:?}"
    );

    let candidates = slot_candidates(&state, garrison_id, fortify_idx, p1);
    assert_eq!(candidates.len(), 1);
    assert!(
        candidates[0].contains(&Target::Object(p1_land_id)),
        "the activating player's own land must be offered; slot: {:?}",
        candidates[0]
    );
    assert!(
        !candidates[0].contains(&Target::Object(p2_land_id)),
        "CR 702.67a 'land YOU control': an opponent's land must not be offered; slot: {:?}",
        candidates[0]
    );
    assert!(
        !candidates[0].contains(&Target::Object(p1_creature_id)),
        "a CREATURE must not be offered — Fortify attaches to a land. (This is the assertion \
         that discriminates the correct `TargetPermanentWithFilter(Land + You)` from a \
         copy-paste of the equip repair's `TargetCreatureWithFilter`, which would have \
         offered exactly this creature and no land at all.) Slot: {:?}",
        candidates[0]
    );
}

/// CR 702.67a end-to-end: the fortification attaches to the targeted land and the
/// printed "fortified land has indestructible" static (EffectFilter::AttachedLand)
/// reaches the land's layer-resolved keywords — which, like Bone Saw's +1/+0, had
/// matched nothing for as long as nothing could attach.
#[test]
fn t8_darksteel_garrison_fortify_attaches_and_grants_indestructible() {
    let (state, garrison_id, p1_land_id, _p2_land, p1_creature_id, p1, p2) = setup_fortify();

    assert!(
        !chars_of(&state, p1_land_id)
            .keywords
            .contains(&KeywordAbility::Indestructible),
        "sanity: the land is not indestructible before the fortification attaches"
    );

    // A creature is not a legal target even if a client names it directly.
    let fortify_idx = fortify_ability_index(&state, garrison_id)
        .expect("Darksteel Garrison must have an activated Fortify ability");
    let creature_attempt = activate(
        state.clone(),
        p1,
        garrison_id,
        fortify_idx,
        vec![Target::Object(p1_creature_id)],
    );
    assert!(
        matches!(creature_attempt, Err(GameStateError::InvalidTarget(_))),
        "CR 702.67a: a creature is not a legal Fortify target; got {:?}",
        creature_attempt.map(|(_, ev)| ev)
    );

    let (state, _) = activate(
        state,
        p1,
        garrison_id,
        fortify_idx,
        vec![Target::Object(p1_land_id)],
    )
    .expect("Fortify {3} on the controller's own land must succeed");
    let (state, events) = pass_all(state, &[p1, p2]);

    assert_eq!(
        state
            .objects()
            .get(&garrison_id)
            .expect("garrison exists")
            .attached_to,
        Some(p1_land_id),
        "Darksteel Garrison must be attached to the targeted land"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::FortificationAttached { .. })),
        "a FortificationAttached event must be emitted; got {events:?}"
    );
    assert!(
        chars_of(&state, p1_land_id)
            .keywords
            .contains(&KeywordAbility::Indestructible),
        "'Fortified land has indestructible' must now apply — the static is filtered by \
         EffectFilter::AttachedLand, so before OOS-CARDS1-1 was closed it matched nothing"
    );
}

// ── T9: guardian_project's nontoken half (OOS-DX3b-1), both directions ─────────

/// **One flow, both directions.** A creature card cast from hand is nontoken and a
/// token it then creates is not, so a single cast exercises the fire-on-match and
/// no-fire-on-mismatch halves of `is_nontoken` against the SAME Guardian Project,
/// with the library count as the observable (a draw is a library of one fewer).
///
/// `OOS-DX3b-1`'s (a) half: the filter's `is_nontoken` is checked by
/// `rules/abilities.rs`'s creature-ETB dispatch *before* matching
/// (`creature_filter.is_nontoken && entering_obj.is_token`) — `is_token` is a
/// runtime `GameObject` field `matches_filter` itself cannot see, which is why the
/// pre-check exists — so authoring `is_nontoken: true` needed zero engine lines.
/// PB-DX3b re-verified that and deferred applying it; PB-DX26 applies it.
///
/// Reverting `guardian_project.rs`'s `is_nontoken: true` makes the SECOND assertion
/// fail (the token draws a card it must not); a filter that refused everything
/// would make the FIRST fail. Neither half passes vacuously.
#[test]
fn t9_guardian_project_draws_on_a_nontoken_etb_and_not_on_a_token_one() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    // A creature card whose own ETB creates a creature TOKEN. Casting it makes a
    // nontoken creature enter (Guardian Project must fire); its trigger then makes a
    // token creature enter (Guardian Project must NOT fire).
    let creator = CardDefinition {
        card_id: mtg_engine::CardId("dx26-token-creator".to_string()),
        name: "DX26 Token Creator".to_string(),
        mana_cost: Some(mtg_engine::ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [mtg_engine::CardType::Creature].iter().cloned().collect(),
            ..Default::default()
        },
        oracle_text: "When this enters, create a 1/1 Soldier creature token.".to_string(),
        abilities: vec![mtg_engine::AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: mtg_engine::TriggerCondition::WhenEntersBattlefield,
            effect: Effect::CreateToken {
                spec: mtg_engine::TokenSpec {
                    name: "Soldier".to_string(),
                    card_types: [mtg_engine::CardType::Creature].into_iter().collect(),
                    subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                    power: 1,
                    toughness: 1,
                    count: mtg_engine::EffectAmount::Fixed(1),
                    ..Default::default()
                },
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = CardRegistry::new({
        let mut v = all_cards();
        v.push(creator.clone());
        v
    });

    let project = enrich_spec_from_def(
        ObjectSpec::card(p1, "Guardian Project")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Guardian Project")),
        &defs,
    );
    let creator_in_hand = ObjectSpec::creature(p1, "DX26 Token Creator", 2, 2)
        .with_card_id(mtg_engine::CardId("dx26-token-creator".to_string()))
        .with_mana_cost(mtg_engine::ManaCost {
            generic: 2,
            ..Default::default()
        })
        .in_zone(ZoneId::Hand(p1));

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(project)
        .object(creator_in_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for i in 0..8 {
        builder = builder.object(
            ObjectSpec::card(p1, &format!("DX26 Library {i}")).in_zone(ZoneId::Library(p1)),
        );
    }
    let mut state = builder.build().unwrap();
    state.turn_mut().priority_holder = Some(p1);

    let library_size = |s: &GameState| -> usize {
        s.objects()
            .values()
            .filter(|o| o.zone == ZoneId::Library(p1))
            .count()
    };
    let before = library_size(&state);
    assert_eq!(before, 8, "sanity: the library starts with 8 cards");

    // Cast the creator: a NONTOKEN creature enters.
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    let creator_card = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == "DX26 Token Creator" && o.zone == ZoneId::Hand(p1))
        .map(|(id, _)| *id)
        .expect("the creator must be in hand");
    let (state, _) = process_command(
        state,
        Command::CastSpell(Box::new(mtg_engine::CastSpellData {
            player: p1,
            card: creator_card,
            targets: vec![],
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
        })),
    )
    .expect("casting the token creator must succeed");

    // Resolve the spell (creature enters), then the two triggers it queued.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    let after_nontoken = library_size(&state);
    assert_eq!(
        after_nontoken,
        before - 1,
        "Guardian Project must draw exactly one card when a NONTOKEN creature you control \
         enters. A filter that refuses everything is not a fix — this is the half that stops \
         `is_nontoken: true` from being read as 'never fire'. Library {before} -> \
         {after_nontoken}"
    );

    // The creator's own ETB has now put a TOKEN creature onto the battlefield.
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.is_token && o.zone == ZoneId::Battlefield),
        "sanity: the creator's ETB must actually have produced a token for the no-fire half \
         to be testing anything"
    );

    // Give the token's ETB every chance to resolve a Guardian Project trigger.
    let (state, _) = pass_all(state, &[p1, p2]);
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        library_size(&state),
        after_nontoken,
        "CR 111.1 / the printed 'whenever a NONTOKEN creature you control enters': Guardian \
         Project must NOT draw when a TOKEN creature enters. A further draw here means the \
         trigger's TargetFilter lost its `is_nontoken: true` (OOS-DX3b-1) — reverting that \
         one field is exactly what makes this assertion fail."
    );
}

// ── T10/T11: the two equip costs that are not plain generic (review Finding L12) ──

/// `glimmer_lens` is the only COLOURED equip cost in the corpus — `{1}{W}`, not
/// `{1}`. Before this probe it was covered only by R7's static comparison against the
/// Scryfall fixture, which reads the def rather than running it: a `ManaCost` whose
/// `white` field never reached the payment path would still have compared equal.
/// This pays it for real, and proves the colour requirement is enforced by paying
/// with two generic first and watching that fail.
#[test]
fn t10_glimmer_lens_equip_requires_its_white_pip() {
    let (state, lens_id, p1_creature_id, _p2, p1, p2) = setup("Glimmer Lens", 0);
    let idx =
        equip_ability_index(&state, lens_id).expect("Glimmer Lens must have an equip ability");

    // Two colourless mana cannot pay {1}{W} (CR 202.1 — the pip is coloured).
    let mut wrong = state.clone();
    wrong
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    assert!(
        activate(
            wrong,
            p1,
            lens_id,
            idx,
            vec![Target::Object(p1_creature_id)]
        )
        .is_err(),
        "Equip {{1}}{{W}} must not be payable with two colourless — the printed cost's white \
         pip is the one thing that distinguishes this def's cost from every other member's"
    );

    // {1} + {W} pays it.
    let mut right = state;
    {
        let pool = &mut right.players_mut().get_mut(&p1).unwrap().mana_pool;
        pool.add(ManaColor::Colorless, 1);
        pool.add(ManaColor::White, 1);
    }
    let (right, _) = activate(
        right,
        p1,
        lens_id,
        idx,
        vec![Target::Object(p1_creature_id)],
    )
    .expect("Equip {1}{W} must be payable with one generic and one white");
    let (right, _) = pass_all(right, &[p1, p2]);
    assert_eq!(
        right
            .objects()
            .get(&lens_id)
            .expect("lens exists")
            .attached_to,
        Some(p1_creature_id),
        "Glimmer Lens must attach once its coloured equip cost is actually paid"
    );
}

/// `umbral_mantle`'s printed line is **Equip {0}** — an empty cost, which is legal and
/// is the one member whose cost could be silently "missing" rather than wrong. A
/// `ManaCost::default()` and a dropped `cost` field look identical in a static
/// comparison; only activating with an empty pool distinguishes them.
#[test]
fn t11_umbral_mantle_equip_costs_nothing_and_still_attaches() {
    let (state, mantle_id, p1_creature_id, _p2, p1, p2) = setup("Umbral Mantle", 0);
    let idx =
        equip_ability_index(&state, mantle_id).expect("Umbral Mantle must have an equip ability");

    let pool = &state.players().get(&p1).unwrap().mana_pool;
    assert_eq!(
        pool.colorless + pool.white + pool.blue + pool.black + pool.red + pool.green,
        0,
        "sanity: the pool is empty, so a successful activation proves the cost really is \
         zero"
    );

    let (state, _) = activate(
        state,
        p1,
        mantle_id,
        idx,
        vec![Target::Object(p1_creature_id)],
    )
    .expect("Equip {0} must be activatable with an empty mana pool");
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state
            .objects()
            .get(&mantle_id)
            .expect("mantle exists")
            .attached_to,
        Some(p1_creature_id),
        "Umbral Mantle must attach — its printed Equip {{0}} costs nothing"
    );
}
