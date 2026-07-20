//! Tests for PB-EF8: `Cost::ExileSelfFromHand` (+ decorative `ActivationZone::Hand`).
//!
//! Simian Spirit Guide / Elvish Spirit Guide each print "Exile this card from your
//! hand: Add {mana}." By CR 605.1a this IS a mana ability (no target, could add mana,
//! not a loyalty ability). Per CR 605.3b it must resolve stacklessly, and per CR 605.5
//! it must not reset priority or `players_passed`. The engine routes it through the
//! same `mana_ability_lowering` -> `handle_tap_for_mana` path as any other mana
//! ability, never `handle_activate_ability`. See
//! `memory/primitives/pb-plan-EF8.md` for the full design.
//!
//! Pattern follows `primitive_sr34_composite_mana_costs.rs`: build the source object
//! directly, activate via `Command::TapForMana`, assert the mana that comes out and
//! the resulting zone/ObjectId state — never just that a `ManaAbility` exists.

use std::collections::HashMap;

use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, AbilityDefinition,
    CardDefinition, CardRegistry, Command, Cost, Effect, GameEvent, GameState, GameStateBuilder,
    GameStateError, ManaColor, ObjectId, ObjectSpec, PlayerId, Step, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn defs_map() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn make_spec(
    owner: PlayerId,
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name)
            .in_zone(zone)
            .with_card_id(card_name_to_id(name)),
        defs,
    )
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn pool_amount(state: &GameState, player: PlayerId, color: ManaColor) -> u32 {
    let pool = &state.player(player).expect("player exists").mana_pool;
    match color {
        ManaColor::White => pool.white,
        ManaColor::Blue => pool.blue,
        ManaColor::Black => pool.black,
        ManaColor::Red => pool.red,
        ManaColor::Green => pool.green,
        ManaColor::Colorless => pool.colorless,
    }
}

/// Build a two-player state with one object (by name) placed in `zone`, priority held
/// by p(1), at pre-combat main.
fn build_with_object_in_zone(
    name: &str,
    zone: ZoneId,
    defs: &HashMap<String, CardDefinition>,
) -> GameState {
    let registry = CardRegistry::new(all_cards());
    let spec = make_spec(p(1), name, zone, defs);
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(registry)
        .object(spec)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .expect("state should build");
    state.turn_mut().priority_holder = Some(p(1));
    state
}

// ── T1 / T2 — happy path (CR 605.1a / 118 / 400.7) ────────────────────────────────

/// CR 605.1a/118/400.7: Simian Spirit Guide activates from hand, exiles the source,
/// adds {R}, and never touches the stack (CR 605.3b).
#[test]
fn simian_activates_from_hand_and_exiles_the_source() {
    let defs = defs_map();
    let state = build_with_object_in_zone("Simian Spirit Guide", ZoneId::Hand(p(1)), &defs);
    let source = find_by_name(&state, "Simian Spirit Guide");

    let (state, events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 0,

            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("Simian Spirit Guide should activate from hand (CR 605.1a)");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Red),
        1,
        "Simian Spirit Guide adds {{R}}"
    );
    assert!(
        state.objects().get(&source).is_none(),
        "CR 400.7: the source ObjectId is dead after the exile"
    );
    let exiled = state
        .objects()
        .iter()
        .filter(|(_, obj)| {
            obj.characteristics.name == "Simian Spirit Guide" && obj.zone == ZoneId::Exile
        })
        .count();
    assert_eq!(
        exiled, 1,
        "exactly one new object must exist in ZoneId::Exile with the card's name"
    );
    assert!(
        state.stack_objects().is_empty(),
        "CR 605.3b: a mana ability must not use the stack"
    );
    assert!(
        events.iter().any(|e| matches!(
            e,
            GameEvent::ObjectExiled { object_id, .. } if *object_id == source
        )),
        "an ObjectExiled{{object_id: source, ..}} event should be emitted"
    );
}

/// Locks the second flip and its colour (CR 605.1a).
#[test]
fn elvish_activates_from_hand_adds_green() {
    let defs = defs_map();
    let state = build_with_object_in_zone("Elvish Spirit Guide", ZoneId::Hand(p(1)), &defs);
    let source = find_by_name(&state, "Elvish Spirit Guide");

    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 0,

            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("Elvish Spirit Guide should activate from hand (CR 605.1a)");

    assert_eq!(
        pool_amount(&state, p(1), ManaColor::Green),
        1,
        "Elvish Spirit Guide adds {{G}}"
    );
    assert!(
        state.objects().get(&source).is_none(),
        "CR 400.7: the source ObjectId is dead after the exile"
    );
}

// ── T3 — stackless invariant (CR 605.5) ───────────────────────────────────────────

/// CR 605.5: activating a mana ability is a special action — it does not reset
/// priority or `players_passed`, even from the from-hand branch.
#[test]
fn from_hand_mana_ability_does_not_reset_priority_or_players_passed() {
    let defs = defs_map();
    let mut state = build_with_object_in_zone("Simian Spirit Guide", ZoneId::Hand(p(1)), &defs);
    state.turn_mut().players_passed.insert(p(2));
    let source = find_by_name(&state, "Simian Spirit Guide");

    let (state, _events) = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 0,

            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    )
    .expect("activation should succeed");

    assert!(
        state.turn().players_passed.contains(&p(2)),
        "CR 605.5: players_passed must be unchanged by a mana ability activation"
    );
    assert_eq!(
        state.turn().players_passed.len(),
        1,
        "players_passed must contain exactly the pre-existing entry, nothing added"
    );
    assert_eq!(
        state.turn().priority_holder,
        Some(p(1)),
        "CR 605.5: the activating player retains priority"
    );
}

// ── T4 / T5 — decoys (both directions of the zone check) ─────────────────────────

/// Decoy A: the same card on the BATTLEFIELD cannot use its from-hand mana ability.
/// Non-vacuity: deleting the `if obj.zone != ZoneId::Hand(player)` check in the
/// `exile_self_from_hand` branch of `handle_tap_for_mana` makes this test pass (the
/// ability would proceed to exile a battlefield permanent and add {R}) — this
/// assertion pins exactly that check.
#[test]
fn decoy_a_same_card_on_battlefield_cannot_use_from_hand_ability() {
    let defs = defs_map();
    let state = build_with_object_in_zone("Simian Spirit Guide", ZoneId::Battlefield, &defs);
    let source = find_by_name(&state, "Simian Spirit Guide");
    let probe_state = state.clone();

    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 0,

            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    );

    assert!(
        matches!(result, Err(GameStateError::InvalidCommand(ref msg)) if msg.contains("from hand")),
        "a battlefield Simian Spirit Guide must be rejected by the from-hand zone check: {result:?}"
    );
    assert!(
        matches!(probe_state.objects()[&source].zone, ZoneId::Battlefield),
        "the source must still be on the battlefield in the caller's pre-command state"
    );
    assert_eq!(
        pool_amount(&probe_state, p(1), ManaColor::Red),
        0,
        "no mana should have been produced"
    );
}

/// Decoy B: a battlefield-only mana ability (a basic Forest — `requires_tap: true`,
/// `exile_self_from_hand: false`) cannot be activated while sitting in hand.
/// Non-vacuity: deleting the `else`-branch `if obj.zone != ZoneId::Battlefield` check
/// would let a hand-zone Forest produce mana; this pins that check.
#[test]
fn decoy_b_battlefield_only_mana_ability_cannot_be_activated_from_hand() {
    let defs = defs_map();
    let state = build_with_object_in_zone("Forest", ZoneId::Hand(p(1)), &defs);
    let source = find_by_name(&state, "Forest");
    let probe_state = state.clone();

    let result = process_command(
        state,
        Command::TapForMana {
            player: p(1),
            source,
            ability_index: 0,

            chosen_color: None,
                hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
},
    );

    assert!(
        matches!(result, Err(GameStateError::ObjectNotOnBattlefield(id)) if id == source),
        "a hand-zone Forest must be rejected by the battlefield check: {result:?}"
    );
    assert!(
        matches!(probe_state.objects()[&source].zone, ZoneId::Hand(pid) if pid == p(1)),
        "the Forest must still be in hand in the caller's pre-command state"
    );
    assert_eq!(
        pool_amount(&probe_state, p(1), ManaColor::Green),
        0,
        "no mana should have been produced"
    );
}

// ── T6 — the lowering gate is not vacuous ─────────────────────────────────────────

/// (a) A synthetic def with a single `Cost::ExileSelfFromHand` + `Effect::AddMana{R}`
/// lowers to a `ManaAbility` (`exile_self_from_hand == true`, `requires_tap == false`)
/// and is excluded from `activated_abilities`. (b) A synthetic def with
/// `Cost::Sequence([DiscardCard, ExileSelfFromHand])` does NOT lower — `DiscardCard`
/// needs a caller-supplied card, so the whole sequence declines. This proves the
/// no-tap-guard relaxation is scoped to the flag, not to all no-tap costs.
#[test]
fn pb_ef8_lowering_gate_is_not_vacuous() {
    let defs = defs_map();

    // Positive: bare Cost::ExileSelfFromHand lowers.
    let exile_only_def = CardDefinition {
        card_id: mtg_engine::CardId("pb-ef8-vacuity-exile-only".to_string()),
        name: "PB-EF8 Vacuity Probe Exile Only".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::ExileSelfFromHand,
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    red: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    let mut defs_pos = defs.clone();
    defs_pos.insert(exile_only_def.name.clone(), exile_only_def.clone());
    let spec_pos = enrich_spec_from_def(
        ObjectSpec::card(p(1), &exile_only_def.name)
            .in_zone(ZoneId::Hand(p(1)))
            .with_card_id(exile_only_def.card_id.clone()),
        &defs_pos,
    );
    assert_eq!(
        spec_pos.mana_abilities.len(),
        1,
        "a bare Cost::ExileSelfFromHand ability must lower into a ManaAbility (positive control)"
    );
    assert!(
        spec_pos.activated_abilities.is_empty(),
        "the same ability must not ALSO appear in activated_abilities (SF-6 exclusion)"
    );
    assert!(
        spec_pos.mana_abilities[0].exile_self_from_hand,
        "the lowered ManaAbility must carry exile_self_from_hand == true"
    );
    assert!(
        !spec_pos.mana_abilities[0].requires_tap,
        "a from-hand exile-self ability has no {{T}} component"
    );

    // Negative: Cost::Sequence([DiscardCard, ExileSelfFromHand]) must NOT lower —
    // DiscardCard needs a caller-supplied card, which TapForMana has no payload for.
    let discard_exile_def = CardDefinition {
        card_id: mtg_engine::CardId("pb-ef8-vacuity-discard-exile".to_string()),
        name: "PB-EF8 Vacuity Probe Discard Exile".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![Cost::DiscardCard, Cost::ExileSelfFromHand]),
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    red: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    let mut defs_neg = defs;
    defs_neg.insert(discard_exile_def.name.clone(), discard_exile_def.clone());
    let spec_neg = enrich_spec_from_def(
        ObjectSpec::card(p(1), &discard_exile_def.name)
            .in_zone(ZoneId::Hand(p(1)))
            .with_card_id(discard_exile_def.card_id.clone()),
        &defs_neg,
    );
    assert_eq!(
        spec_neg.mana_abilities.len(),
        0,
        "Cost::Sequence([DiscardCard, ExileSelfFromHand]) must NOT lower into a \
         ManaAbility (negative control — DiscardCard needs a caller-supplied card)"
    );
    assert_eq!(
        spec_neg.activated_abilities.len(),
        1,
        "it must instead register as a stack-using activated ability"
    );

    // Negative #2 (direct scoping control): a bare `Cost::SacrificeSelf` no-tap mana
    // ability (Food Chain shape — no {T}, no exile-from-hand) must STILL decline. Its
    // only disqualifier is the no-tap guard, so this pins that the guard's relaxation is
    // gated on `exile_self_from_hand` alone and does NOT lower every no-tap cost (CR
    // 605.1a / SR-34). If the relaxation were widened to `!acc.requires_tap` outright,
    // this would wrongly lower into a free, repeatable, stackless `Add {R}`.
    let sac_only_def = CardDefinition {
        card_id: mtg_engine::CardId("pb-ef8-vacuity-sac-only".to_string()),
        name: "PB-EF8 Vacuity Probe Sacrifice Only".to_string(),
        types: mtg_engine::cards::card_definition::TypeLine {
            card_types: vec![mtg_engine::CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::SacrificeSelf,
            effect: Effect::AddMana {
                player: mtg_engine::cards::card_definition::PlayerTarget::Controller,
                mana: mtg_engine::ManaPool {
                    red: 1,
                    ..Default::default()
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    };
    let mut defs_sac = defs_neg;
    defs_sac.insert(sac_only_def.name.clone(), sac_only_def.clone());
    let spec_sac = enrich_spec_from_def(
        ObjectSpec::card(p(1), &sac_only_def.name)
            .in_zone(ZoneId::Battlefield)
            .with_card_id(sac_only_def.card_id.clone()),
        &defs_sac,
    );
    assert_eq!(
        spec_sac.mana_abilities.len(),
        0,
        "a bare no-tap Cost::SacrificeSelf mana ability must NOT lower (the no-tap guard \
         relaxation is scoped to exile_self_from_hand only — CR 605.1a / SR-34)"
    );
    assert_eq!(
        spec_sac.activated_abilities.len(),
        1,
        "it must instead register as a stack-using activated ability"
    );
}

// ── T7 — CR 106.12: exiling from hand is not tapping ─────────────────────────────

/// CR 106.12: "To 'tap [a permanent] for mana' is to activate a mana ability of that
/// permanent that includes the {T} symbol in its activation cost." The Spirit Guides
/// have no {T} — this asserts the narrower, cheap-to-check invariant that makes the
/// CR 106.12 reading correct by construction: `requires_tap == false`, so
/// `handle_tap_for_mana`'s `requires_tap`-gated steps (mana-production replacements
/// like Nyxbloom Ancient, and `WhenTappedForMana` triggers) are structurally
/// unreachable for this ability.
#[test]
fn from_hand_mana_ability_does_not_fire_tapped_for_mana_replacements() {
    let defs = defs_map();
    let spec = make_spec(p(1), "Simian Spirit Guide", ZoneId::Hand(p(1)), &defs);
    assert_eq!(spec.mana_abilities.len(), 1);
    assert!(
        !spec.mana_abilities[0].requires_tap,
        "CR 106.12: exiling from hand does not tap, so requires_tap must be false — this \
         is what makes the requires_tap-gated mana-production-replacement and \
         WhenTappedForMana-trigger steps in handle_tap_for_mana structurally unreachable \
         for this ability"
    );
}
