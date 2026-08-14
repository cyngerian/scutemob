//! CARDS-1 (OOS-M11-10): equip activations silently fizzle when the ability
//! declares zero `TargetRequirement`s.
//!
//! Chain (verified by symbol, not by claim):
//!   1. `handle_activate_ability` (`rules/abilities.rs:315-334`) reads
//!      `target_requirements` from the layer-resolved `ActivatedAbility.targets`.
//!   2. `rules/abilities.rs:495` — general CR 601.2c target validation only runs
//!      `if !target_requirements.is_empty()`. All 16 hand-authored equip abilities
//!      (every `Effect::AttachEquipment` def except Helm of the Host) declare
//!      `targets: vec![]`, so a zero-target activation is silently ACCEPTED.
//!   3. `rules/abilities.rs:539-582` — a legacy special-case validates a
//!      *volunteered* target via `targets.first()`. With zero targets this `if let`
//!      is a no-op; its own comment says proper `TargetRequirement` declarations are
//!      "validated by the general check above" (step 2), which never ran.
//!   4. At resolution, `Effect::AttachEquipment`'s `EffectTarget::DeclaredTarget {
//!      index: 0 }` resolves against an empty target list -> nothing -> the attach
//!      is silently skipped (`effects/mod.rs:5213`+, the `for target_res in
//!      &target_resolved` loop simply never iterates). Cost was already paid.
//!
//! The fix (applied separately, NOT by this file) gives all **17** roster members
//! `targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
//! controller: TargetController::You, ..Default::default() })]` (CR 702.6a: "Attach
//! this permanent to target creature you control"). Seventeen, not sixteen: the 16
//! above are the ones that declared NOTHING, but Helm of the Host — the def the seed
//! row called the one that "declares the `TargetRequirement`" — declared a bare
//! `TargetRequirement::TargetCreature`, dropping 702.6a's "you control" clause, so
//! its repair is a tightening rather than an addition. Being the only member with a
//! requirement was not the same as being the only member with a correct one.
//!
//! T1 is a PERMANENT record of the pre-fix defect shape (built synthetically, not
//! via editing a real card def) and must keep passing after the fix — it proves the
//! defect existed, not that it still does. T2-T6 exercise the real Skullclamp def
//! (`crates/card-defs/src/defs/skullclamp.rs`, ability index 0, Equip {1}) and are
//! the discriminating half: they FAIL against the pre-fix card corpus and must PASS
//! once the roster is repaired. Measured pre-fix, not predicted: T2, T5 and T6 failed
//! and T3/T4 already passed, because step 3's legacy special-case does validate a
//! target that IS volunteered — the defect is that nothing ever asks for one, which
//! is exactly why the browser client surfaced this and the TUI never did. Verbatim
//! pre-fix output: `memory/primitives/cards1-equip-fail-before-2026-08-02.md`.

use mtg_engine::state::{ActivatedAbility, ActivationCost, GameStateError};
use mtg_engine::{
    ability_target_requirements, all_cards, calculate_characteristics, card_name_to_id,
    enrich_spec_from_def, legal_targets_per_slot, process_command, CardDefinition,
    CardEffectTarget, CardRegistry, Command, Effect, GameEvent, GameState, GameStateBuilder,
    ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, SubType, Target, TargetController,
    TargetFilter, TargetRequirement, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_object_controlled_by(state: &GameState, name: &str, controller: PlayerId) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.controller == controller)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' controlled by {:?} not found", name, controller))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
}

/// Build an `ActivatedAbility` with the EXACT pre-fix shape of the 16 hand-authored
/// equip defs: `targets: vec![]` and `Effect::AttachEquipment` reading a declared
/// target that will never be validated to exist. This is how the plan asks T1 to be
/// built: via the `ActivatedAbility` route, NOT by editing a card def.
fn broken_equip_ability(generic_mana: u32) -> ActivatedAbility {
    ActivatedAbility {
        targets: vec![], // <- the defect: no TargetRequirement declared
        cost: ActivationCost {
            requires_tap: false,
            mana_cost: Some(ManaCost {
                generic: generic_mana,
                ..Default::default()
            }),
            sacrifice_self: false,
            discard_card: false,
            discard_self: false,
            forage: false,
            sacrifice_filter: None,
            remove_counter_cost: None,
            exile_self: false,
            exert: false,
            life_cost: 0,
            sacrifice_exclude_self: false,
            exile_self_from_hand: false,
        },
        description: format!("Equip {{{}}} (pre-fix shape)", generic_mana),
        effect: Some(Effect::AttachEquipment {
            equipment: CardEffectTarget::Source,
            target: CardEffectTarget::DeclaredTarget { index: 0 },
        }),
        sorcery_speed: true,
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: None,
    }
}

/// Common two-player setup: real Skullclamp (ability index 0, Equip {1}) plus a
/// creature controlled by p1 and a creature controlled by p2, p1 has priority with
/// exactly 1 generic mana in pool.
fn setup_skullclamp_scenario() -> (GameState, ObjectId, ObjectId, ObjectId, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let skullclamp = enrich_spec_from_def(
        ObjectSpec::card(p1, "Skullclamp")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(card_name_to_id("Skullclamp")),
        &defs,
    );
    let p1_creature = ObjectSpec::creature(p1, "P1 Bear", 2, 2);
    let p2_creature = ObjectSpec::creature(p2, "P2 Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(all_cards()))
        .object(skullclamp)
        .object(p1_creature)
        .object(p2_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);

    let skullclamp_id = find_object(&state, "Skullclamp");
    let p1_creature_id = find_object_controlled_by(&state, "P1 Bear", p1);
    let p2_creature_id = find_object_controlled_by(&state, "P2 Bear", p2);

    // `GameStateBuilder::object()` places Skullclamp directly on the battlefield --
    // it does NOT run the ETB machinery that normally calls
    // `replacement::register_static_continuous_effects` (that only happens along
    // the real cast/resolve/land-play paths, `rules/resolution.rs`/`rules/lands.rs`).
    // Without this, Skullclamp's two `AbilityDefinition::Static` entries (+1 power,
    // -1 toughness, both `EffectFilter::AttachedCreature`) would never be in
    // `state.continuous_effects`, and T3's layer-resolved P/T assertion would fail
    // for a reason unrelated to OOS-M11-10 -- mirrors `equip.rs`'s
    // `test_equip_grants_keywords_via_layer_system`, which manually injects the
    // equivalent `ContinuousEffect` for its synthetic equipment.
    let skullclamp_card_id = state
        .objects()
        .get(&skullclamp_id)
        .and_then(|o| o.card_id.clone());
    let registry = state.card_registry().clone();
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        skullclamp_id,
        skullclamp_card_id.as_ref(),
        &registry,
        false,
    );

    (state, skullclamp_id, p1_creature_id, p2_creature_id, p1, p2)
}

// ── T1: pre-fix shape reproduction (permanent defect record) ──────────────────

/// CR 702.6a / CR 601.2c — an activated ability with `Effect::AttachEquipment` and
/// `targets: vec![]` (the pre-fix shape of all 16 hand-authored equip defs) is
/// ACCEPTED with zero declared targets, pays its cost, and resolves as a silent
/// no-op: no attach happens and no event signals anything went wrong. This is the
/// "cost paid, silent fizzle" proof; it is expected to remain true forever (it is
/// exercising a synthetic ability, not a real card def) and is a permanent
/// regression guard for the shape itself, not for whether any specific card still
/// has it.
#[test]
fn t1_zero_target_ability_accepted_paid_and_silently_fizzles() {
    let p1 = p(1);
    let p2 = p(2);

    let equipment = ObjectSpec::artifact(p1, "Broken Sword")
        .with_subtypes(vec![SubType("Equipment".to_string())])
        .with_activated_ability(broken_equip_ability(2));
    let creature = ObjectSpec::creature(p1, "Target Bear", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(equipment)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn_mut().priority_holder = Some(p1);

    let equip_id = find_object(&state, "Broken Sword");
    let creature_id = find_object(&state, "Target Bear");

    // (a) The activation with ZERO declared targets is ACCEPTED.
    let (state, activate_events) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: equip_id,
            ability_index: 0,
            targets: vec![], // no target declared -- the ability requires none
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect(
        "BUG DEMONSTRATED: a zero-target activation of an AttachEquipment ability with \
             targets: vec![] must be accepted pre-fix (this IS the defect)",
    );

    // (b) The mana cost was actually paid -- not free.
    assert!(
        activate_events
            .iter()
            .any(|e| matches!(e, GameEvent::ManaCostPaid { player, .. } if *player == p1)),
        "cost must be paid even though the activation will silently do nothing"
    );
    let mana_pool = &state.players().get(&p1).unwrap().mana_pool;
    let total_mana = mana_pool.colorless
        + mana_pool.white
        + mana_pool.blue
        + mana_pool.black
        + mana_pool.red
        + mana_pool.green;
    assert_eq!(total_mana, 0, "the 2 generic mana must have been spent");

    // (c) After resolving, the Equipment is attached to NOTHING.
    let (state, resolve_events) = pass_all(state, &[p1, p2]);
    let equip_obj = state.objects().get(&equip_id).expect("equipment exists");
    assert_eq!(
        equip_obj.attached_to, None,
        "the equipment must NOT be attached to anything -- the target never resolved"
    );
    let creature_obj = state.objects().get(&creature_id).expect("creature exists");
    assert!(
        !creature_obj.attachments.contains(&equip_id),
        "the creature must not have gained the equipment"
    );

    // (d) No error-shaped event was emitted that a player would notice. There is no
    // ability-fizzle event in the enum at all (only `SpellFizzled`, which applies to
    // spells, not abilities) -- `AbilityResolved` fires unconditionally, carrying no
    // success/failure signal. This IS the bug: the absence of any negative signal.
    assert!(
        resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityResolved { .. })),
        "the ability resolves normally (silently) -- AbilityResolved fires with no \
         indication anything went wrong"
    );
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::EquipmentAttached { .. })),
        "no EquipmentAttached event -- the attach never happened"
    );
    assert!(
        !resolve_events
            .iter()
            .any(|e| matches!(e, GameEvent::SpellFizzled { .. })),
        "SpellFizzled does not and cannot apply to an ability; there is no ability-level \
         analogue in the event enum -- this is exactly the diagnosability gap"
    );
}

// ── T2: post-fix -- zero targets against a real def is now REJECTED ───────────

/// CR 601.2c / CR 702.6a -- once Skullclamp's equip ability declares its
/// `TargetRequirement`, activating it with zero declared targets must be REJECTED,
/// not silently accepted. This is the discriminating half: it FAILS against the
/// pre-fix card corpus (Skullclamp currently declares `targets: vec![]`) and must
/// PASS once the fix lands.
#[test]
fn t2_skullclamp_zero_targets_rejected_post_fix() {
    let (state, skullclamp_id, _p1_creature_id, _p2_creature_id, p1, _p2) =
        setup_skullclamp_scenario();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: skullclamp_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    match &result {
        Err(GameStateError::InvalidTarget(msg)) => {
            // `validate_targets_inner` (casting.rs) computes (min_t, max_t) = (1, 1)
            // from a single mandatory TargetCreatureWithFilter requirement, then
            // rejects because `targets.len() (0) < min_t (1)` with exactly this
            // message shape.
            assert!(
                msg.contains("expected 1..=1 target(s) but got 0"),
                "expected the target-count-range rejection message, got: {msg:?}"
            );
        }
        Ok(_) => panic!(
            "expected Err(GameStateError::InvalidTarget(_)) once Skullclamp declares its \
             TargetRequirement (CR 601.2c: 0 declared targets against a 1-target-mandatory \
             requirement is illegal); got Ok(_) instead -- pre-fix, Skullclamp's equip ability \
             still declares targets: vec![] so a zero-target activation is silently ACCEPTED \
             (this IS the bug this batch closes)"
        ),
        Err(other) => panic!(
            "expected Err(GameStateError::InvalidTarget(_)), got a different Err variant: {:?}",
            other
        ),
    }
}

// ── T3: post-fix -- a legal target attaches end-to-end ─────────────────────────

/// CR 702.6a -- targeting a creature the activating player controls succeeds,
/// resolves, attaches, and the equip bonus (+1/-1) is reflected via layer-resolved
/// characteristics.
#[test]
fn t3_skullclamp_legal_target_attaches_and_applies_bonus() {
    let (state, skullclamp_id, p1_creature_id, _p2_creature_id, p1, p2) =
        setup_skullclamp_scenario();

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: skullclamp_id,
            ability_index: 0,
            targets: vec![Target::Object(p1_creature_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect(
        "activating Skullclamp targeting the controller's own creature must succeed \
             post-fix",
    );

    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    let equip_obj = state
        .objects()
        .get(&skullclamp_id)
        .expect("skullclamp exists");
    assert_eq!(
        equip_obj.attached_to,
        Some(p1_creature_id),
        "Skullclamp must be attached to the targeted creature"
    );
    assert!(
        resolve_events.iter().any(|e| matches!(
            e,
            GameEvent::EquipmentAttached { equipment_id, target_id, controller }
            if *equipment_id == skullclamp_id && *target_id == p1_creature_id && *controller == p1
        )),
        "EquipmentAttached event expected"
    );

    // Skullclamp: "Equipped creature gets +1/-1." Base 2/2 -> 3/1 via layer 7c.
    let chars =
        calculate_characteristics(&state, p1_creature_id).expect("creature exists after equipping");
    assert_eq!(
        (chars.power, chars.toughness),
        (Some(3), Some(1)),
        "Skullclamp's +1/-1 static ability must apply via layer-resolved characteristics"
    );
}

// ── T4: post-fix -- an opponent's creature is an illegal target ───────────────

/// CR 702.6a -- "target creature you control." Targeting an opponent's creature is
/// rejected once the TargetRequirement carries `TargetController::You`.
#[test]
fn t4_skullclamp_opponent_creature_rejected_post_fix() {
    let (state, skullclamp_id, _p1_creature_id, p2_creature_id, p1, _p2) =
        setup_skullclamp_scenario();

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: skullclamp_id,
            ability_index: 0,
            targets: vec![Target::Object(p2_creature_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "equip targeting an opponent's creature must be rejected post-fix \
         (CR 702.6a: 'target creature you control'), got {:?}",
        result
    );
}

// ── T5: browser-path -- the engine query now reports the slot ─────────────────

/// The read-only query surface (`rules::queries`, used by `tools/play-server` to
/// populate `ActionOptionView.target_slots`) must report Skullclamp's equip slot
/// once the def declares it, and the per-slot candidate list must be scoped to
/// creatures the activating player controls -- this is the assertion that would
/// have caught "the picker never asks" (the browser-path half of OOS-M11-10).
#[test]
fn t5_engine_query_reports_slot_and_candidates_scoped_to_controller() {
    let (state, skullclamp_id, p1_creature_id, p2_creature_id, p1, _p2) =
        setup_skullclamp_scenario();

    let reqs = ability_target_requirements(&state, skullclamp_id, 0);
    assert_eq!(
        reqs.len(),
        1,
        "Skullclamp's equip ability must report exactly one TargetRequirement post-fix"
    );
    match &reqs[0] {
        TargetRequirement::TargetCreatureWithFilter(filter) => {
            assert_eq!(
                filter,
                &TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                },
                "the requirement must be scoped to the activating player's own creatures \
                 (CR 702.6a) and otherwise unrestricted"
            );
        }
        other => panic!(
            "expected TargetRequirement::TargetCreatureWithFilter, got {:?}",
            other
        ),
    }

    let candidates = legal_targets_per_slot(&state, p1, skullclamp_id, &reqs);
    assert_eq!(candidates.len(), 1, "one slot for one requirement");
    assert!(
        candidates[0].contains(&Target::Object(p1_creature_id)),
        "the controller's own creature must be an offered candidate"
    );
    assert!(
        !candidates[0].contains(&Target::Object(p2_creature_id)),
        "an opponent's creature must NOT be an offered candidate (CR 702.6a 'you control') \
         -- this is the exact assertion that would have caught the browser picker never \
         asking for a target at all (an empty-requirements query never reaches this check)"
    );
}

// ── T6: non-vacuity floors ──────────────────────────────────────────────────────

/// Pinning an empty set is the shape that rots silently (the PB-DX6 R2/R4 lesson).
/// Both halves of T5's candidate proof, and the equip-Activated-AttachEquipment
/// roster itself, must be non-empty.
#[test]
fn t6_non_vacuity_floors() {
    let (state, skullclamp_id, p1_creature_id, _p2_creature_id, p1, _p2) =
        setup_skullclamp_scenario();

    let reqs = ability_target_requirements(&state, skullclamp_id, 0);
    assert!(!reqs.is_empty(), "T5's requirement list must be non-empty");
    let candidates = legal_targets_per_slot(&state, p1, skullclamp_id, &reqs);
    assert!(
        !candidates.is_empty() && !candidates[0].is_empty(),
        "T5's per-slot candidate list must be non-empty"
    );
    assert!(
        candidates[0].contains(&Target::Object(p1_creature_id)),
        "sanity: the non-empty candidate list must contain the expected creature"
    );

    let defs = all_cards();
    let roster = equip_activated_attach_equipment_roster(&defs);
    assert!(
        !roster.is_empty(),
        "the AttachEquipment-carrying Activated-ability roster must be non-empty (39 \
         expected -- see core/cards1_equip_target_roster.rs R1)"
    );
    // PB-DX27 (`scutemob-209`, 2026-08-13): 38 -> 39, and the re-pin is worth explaining
    // because the two gates' "38" turn out to have meant different things.
    // `equip_activated_attach_equipment_roster` pushes `def.name` once per MATCHING
    // ABILITY, so it is an ability count; `core::cards1_equip_target_roster` R1 builds a
    // SET of names, so it is a def count. They agreed at 38 only because every equip def
    // happened to carry exactly one equip ability. `blackblade_reforged` now carries two
    // -- CR 702.6c makes "Equip legendary creature {3}" a SEPARATE activated ability from
    // its Equip {7}, not a second cost on one ability -- so this count moves to 39 while
    // R1 correctly stays at 38. A coincidence between two numbers is not an invariant.
    assert_eq!(
        roster.len(),
        39,
        "expected exactly 39 equip ABILITIES (CARDS-1's 17 + PB-DX26's 21 formerly \
         marker-only defs + PB-DX27's second blackblade_reforged equip ability); note this \
         is an ability count, unlike core::cards1_equip_target_roster R1's def count of 38. \
         Found {}",
        roster.len()
    );
}

// ── T7: Aura / Fortify / Reconfigure untouched ──────────────────────────────────

/// Enumerate `mtg_engine::all_cards()` (SR-36 -- never grep source) for the set of
/// defs carrying an `AbilityDefinition::Activated` whose `effect` is
/// `Effect::AttachEquipment`. This is the SAME shape the core roster test's R1
/// pins; duplicated here (independently derived, not copy-pasted logic reused
/// across test binaries -- `tests/primitives` and `tests/core` are separate
/// binaries and cannot share code without a shared support crate) as this file's
/// own non-vacuity check.
fn equip_activated_attach_equipment_roster(defs: &[CardDefinition]) -> Vec<String> {
    use mtg_engine::AbilityDefinition;
    let mut out = Vec::new();
    for def in defs {
        // PB-DX26 fix cycle (review Finding 6): both faces, matching the census in
        // `core::pb_dx26_attach_keyword_roster` rather than the front-face-only walks
        // this file used to share with it.
        for ability in std::iter::once(&def.abilities)
            .chain(def.back_face.iter().map(|f| &f.abilities))
            .flatten()
        {
            if let AbilityDefinition::Activated { effect, .. } = ability {
                // PB-DX26: was a flat `matches!`, which dropped a Sequence-nested
                // attach out of the pin SILENTLY (`seed-rerank-2026-08-02.md` §2.7
                // names this line by number). Reuses the file's own recursive
                // finder, which PB-DX26 widened past `Sequence` to every nesting
                // site in the `Effect` enum.
                if find_attach_equipment_target(effect).is_some() {
                    out.push(def.name.clone());
                }
            }
        }
    }
    out
}

/// Recursively search an `Effect` tree for an `Effect::AttachEquipment`,
/// returning its `target` field if found.
///
/// **PB-DX26 widened this past `Sequence`.** It walks every `Box<Effect>` /
/// `Vec<Effect>` nesting site in the `Effect` enum; that site list is itself
/// pinned by `core::pb_dx26_attach_keyword_roster::r6`, so this walk fails loudly
/// rather than going quietly shallow when a new nesting variant is added.
fn find_attach_equipment_target(effect: &Effect) -> Option<&CardEffectTarget> {
    match effect {
        Effect::AttachEquipment { target, .. } => Some(target),
        Effect::Sequence(effects) => effects.iter().find_map(find_attach_equipment_target),
        Effect::Conditional {
            if_true, if_false, ..
        } => {
            find_attach_equipment_target(if_true).or_else(|| find_attach_equipment_target(if_false))
        }
        Effect::Repeat { effect, .. } => find_attach_equipment_target(effect),
        Effect::ForEach { effect, .. } => find_attach_equipment_target(effect),
        Effect::Choose { choices, .. } => choices.iter().find_map(find_attach_equipment_target),
        Effect::MayPayOrElse { or_else, .. } => find_attach_equipment_target(or_else),
        Effect::MayPayThenEffect { then, .. } => find_attach_equipment_target(then),
        Effect::CoinFlip {
            on_win, on_lose, ..
        } => find_attach_equipment_target(on_win).or_else(|| find_attach_equipment_target(on_lose)),
        // PB-DX26 fix cycle: `Vec<(u32, u32, Effect)>` — the eleventh site, invisible
        // to a `Box<Effect>`/`Vec<Effect>` count and missed by this walk's first draft.
        Effect::RollDice { results, .. } => results
            .iter()
            .find_map(|(_, _, e)| find_attach_equipment_target(e)),
        _ => None,
    }
}

/// (a) Cryptic Coat's `AttachEquipment` site is a `Triggered` ability targeting
/// `EffectTarget::LastCreatedPermanent` with `targets: vec![]` -- it correctly
/// needs no target (the target is determined by the trigger's own effect, not a
/// player choice) and must stay OUT of the repaired roster. This batch must not
/// have perturbed it.
#[test]
fn t7a_cryptic_coat_triggered_attach_untouched() {
    use mtg_engine::AbilityDefinition;

    let defs = all_cards();
    let roster = equip_activated_attach_equipment_roster(&defs);
    assert!(
        !roster.contains(&"Cryptic Coat".to_string()),
        "Cryptic Coat's AttachEquipment is a Triggered ability, not Activated -- it must \
         NOT be a member of the repaired Activated-ability roster"
    );

    let cryptic_coat = defs
        .iter()
        .find(|d| d.name == "Cryptic Coat")
        .expect("Cryptic Coat must exist in the corpus");

    let mut found_triggered_attach = false;
    for ability in &cryptic_coat.abilities {
        if let AbilityDefinition::Triggered {
            effect, targets, ..
        } = ability
        {
            if let Some(target) = find_attach_equipment_target(effect) {
                found_triggered_attach = true;
                assert!(
                    matches!(target, CardEffectTarget::LastCreatedPermanent),
                    "Cryptic Coat's Triggered AttachEquipment must target \
                     LastCreatedPermanent, got {:?}",
                    target
                );
                assert!(
                    targets.is_empty(),
                    "Cryptic Coat's Triggered ability correctly needs zero declared \
                     TargetRequirements -- LastCreatedPermanent is not a player choice"
                );
            }
        }
    }
    assert!(
        found_triggered_attach,
        "expected to find a Triggered ability on Cryptic Coat whose effect (possibly \
         nested in a Sequence) is Effect::AttachEquipment"
    );
}

/// (b) Pin the set of defs producing `Effect::AttachFortification` (Fortify, CR
/// 702.67a) and the set carrying `KeywordAbility::Reconfigure` (CR 702.151) as
/// EXACT sets, measured by enumerating `all_cards()` with `--nocapture` (see
/// `memory/primitives/cards1-equip-fail-before-2026-08-02.md`), not guessed.
///
/// **What membership here asserts, and does NOT assert**: only that this def's
/// shape exists at this specific site (an Activated+AttachFortification ability,
/// or a `KeywordAbility::Reconfigure` marker). It says nothing about whether that
/// def's own equip-style target validation is itself correct.
///
/// **Reconfigure (Lizard Blades) — CLOSED by PB-DX20.** The `targets: vec![]` defect
/// this comment used to describe is fixed: `testing/replay_harness.rs`'s
/// `AbilityDefinition::Reconfigure` attach-arm synth site now carries CR 702.151a's
/// "another target creature you control" requirement
/// (`TargetCreatureWithFilter { controller: You, exclude_self: true, .. }`), proven
/// through the real corpus synth path (not a hand-built stand-in) by
/// `pb_dx20_keyword_carried_target_requirements.rs`'s T5 probes.
///
/// **Fortify (Darksteel Garrison) — CLOSED by PB-DX26 (`OOS-CARDS1-1`).** It had
/// carried the exact same `targets: vec![]` shape the equip roster had before this
/// batch's predecessor (CARDS-1) fixed Equipment; PB-DX20's scope was Aura +
/// Reconfigure only, so Fortify was in neither plan and this pin was the record of
/// that. It is now fixed, and the pin below is **strengthened rather than deleted**:
/// it no longer only asserts *who* is in the roster, it asserts that the one member
/// declares CR 702.67a's requirement — `TargetPermanentWithFilter(has_card_type
/// Land + controller You)`, explicitly **not** the equip repair's
/// `TargetCreatureWithFilter`, which would demand a creature this ability may never
/// legally attach to. A name-set pin alone would have stayed green through the fix
/// and through a regression of it alike, which is the failure mode this whole file
/// exists to prevent.
#[test]
fn t7b_fortify_and_reconfigure_rosters_pinned_and_unperturbed() {
    use mtg_engine::{
        AbilityDefinition, CardType, KeywordAbility, TargetController, TargetFilter,
        TargetRequirement,
    };
    use std::collections::BTreeSet;

    let defs = all_cards();

    let mut fortify_roster = BTreeSet::new();
    for def in &defs {
        for ability in &def.abilities {
            if let AbilityDefinition::Activated { effect, .. } = ability {
                if matches!(effect, Effect::AttachFortification { .. }) {
                    fortify_roster.insert(def.name.clone());
                }
            }
        }
    }
    println!("t7b measured fortify_roster = {fortify_roster:?}");

    let mut reconfigure_roster = BTreeSet::new();
    for def in &defs {
        for ability in &def.abilities {
            if let AbilityDefinition::Keyword(KeywordAbility::Reconfigure) = ability {
                reconfigure_roster.insert(def.name.clone());
            }
        }
    }
    println!("t7b measured reconfigure_roster = {reconfigure_roster:?}");

    let expected_fortify: BTreeSet<String> = ["Darksteel Garrison"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let expected_reconfigure: BTreeSet<String> =
        ["Lizard Blades"].iter().map(|s| s.to_string()).collect();

    assert_eq!(
        fortify_roster, expected_fortify,
        "Fortify (Effect::AttachFortification) roster changed -- this batch touches \
         Equipment only and must not have added/removed a Fortification def. \
         Found: {fortify_roster:?}, expected: {expected_fortify:?}"
    );
    assert_eq!(
        reconfigure_roster, expected_reconfigure,
        "Reconfigure (KeywordAbility::Reconfigure) roster changed -- this batch touches \
         AbilityDefinition::Activated + Effect::AttachEquipment defs only, never \
         AbilityDefinition::Reconfigure. Found: {reconfigure_roster:?}, expected: \
         {expected_reconfigure:?}"
    );

    // PB-DX26 (OOS-CARDS1-1): the Fortify member's declared requirement, pinned by
    // SHAPE and not merely by name. CR 702.67a is "attach to target LAND you
    // control" -- a different requirement from equip's CR 702.6a creature, and the
    // one place a copy-paste of the equip repair would have gone wrong silently
    // (an activation offering only creatures can never attach a Fortification).
    let expected_fortify_requirement = TargetRequirement::TargetPermanentWithFilter(TargetFilter {
        has_card_type: Some(CardType::Land),
        controller: TargetController::You,
        ..Default::default()
    });
    let mut checked = 0usize;
    for def in &defs {
        for ability in &def.abilities {
            if let AbilityDefinition::Activated {
                effect, targets, ..
            } = ability
            {
                if matches!(effect, Effect::AttachFortification { .. }) {
                    checked += 1;
                    assert_eq!(
                        targets.as_slice(),
                        std::slice::from_ref(&expected_fortify_requirement),
                        "'{}' declares the wrong target requirement for its Fortify ability. \
                         CR 702.67a: '[Cost]: Attach this permanent to target LAND you \
                         control.' An empty `targets` vec is OOS-CARDS1-1's original defect \
                         -- the offer layer reports zero slots, nothing asks, the cost is \
                         paid and the attach fizzles in silence. A `TargetCreatureWithFilter` \
                         here is the copy-the-equip-repair mistake: it demands a creature.\n\
                         Found:    {targets:?}\nExpected: [{expected_fortify_requirement:?}]",
                        def.name
                    );
                }
            }
        }
    }
    assert_eq!(
        checked, 1,
        "non-vacuity: exactly one def must carry an Activated Effect::AttachFortification \
         ability for the requirement check above to have examined anything; found {checked}"
    );
}
