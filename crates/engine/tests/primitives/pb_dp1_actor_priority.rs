//! PB-DP1 probes: priority after casting a spell, activating an ability, or
//! taking a special action goes to the ACTOR, not the active player.
//!
//! CR 117.3c: "If a player has priority when they cast a spell, activate an
//! ability, or take a special action, that player receives priority afterward."
//! CR 116.3: "If a player takes a special action, that player receives
//! priority afterward."
//! CR 117.4: an action taken between passes restarts the pass-round
//! (`players_passed` must be reset).
//!
//! Written FIRST and run RED before any engine line moves (AC 5512). P1-P5 and
//! P8 must fail against the pre-fix engine; P6 and P7 are green-both-sides
//! control probes by design (see `memory/primitives/pb-plan-DP1.md` §3).

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    process_command, AbilityDefinition, ActivatedAbility, ActivationCost, AltCostKind,
    CardDefinition, CardId, CardRegistry, CardType, Command, CounterType, Effect, EffectAmount,
    FaceDownKind, GameState, GameStateBuilder, GameStateError, KeywordAbility, LoyaltyCost,
    ManaAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, SubType,
    TurnFaceUpMethod, ZoneId,
};

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

/// Tap-only activated ability, instant speed (no sorcery-speed restriction).
fn tap_ability(description: &str) -> ActivatedAbility {
    ActivatedAbility {
        targets: vec![],
        cost: ActivationCost {
            requires_tap: true,
            mana_cost: None,
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
        description: description.to_string(),
        effect: None,
        sorcery_speed: false,
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        modes: None,
    }
}

/// A minimal `CastSpellData` with every alt-cost/mode field at its default,
/// so each probe only needs to set `player` and `card`.
fn cast(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
        player,
        card,
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
    }))
}

/// Build a Vehicle ObjectSpec (mirrors `mechanics_a_d/crew.rs::vehicle_spec`).
fn vehicle_spec(
    owner: PlayerId,
    name: &str,
    power: i32,
    toughness: i32,
    crew_n: u32,
) -> ObjectSpec {
    let mut spec = ObjectSpec::artifact(owner, name)
        .with_subtypes(vec![SubType("Vehicle".to_string())])
        .with_keyword(KeywordAbility::Crew(crew_n));
    spec.power = Some(power);
    spec.toughness = Some(toughness);
    spec
}

// ── P1 — non-active player casting an instant retains priority ───────────────

#[test]
/// CR 117.3c, CR 601.2i — a non-active player who holds priority casts an
/// instant and retains priority afterward; it does not revert to the active
/// player.
fn test_dp1_non_active_player_casting_instant_retains_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let instant = ObjectSpec::card(p2, "Probe Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(instant)
        .build()
        .unwrap();
    // Simulate p1 having passed priority to p2 (CR 117.3d).
    state.turn_mut().priority_holder = Some(p2);

    let card_id = find_object(&state, "Probe Bolt");
    let (new_state, _events) = process_command(state, cast(p2, card_id))
        .expect("CR 117.3c: p2 held priority and should be able to cast an instant");

    assert_eq!(
        new_state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: the caster, not the active player, receives priority after casting"
    );
}

// ── P2 — actor can respond to their own spell with no intervening pass ───────

#[test]
/// CR 117.3c — after casting, the actor retains priority and may immediately
/// cast a second spell in response to their own, with no intervening
/// `PassPriority`.
fn test_dp1_actor_can_respond_to_own_spell() {
    let p1 = p(1);
    let p2 = p(2);

    let instant_a = ObjectSpec::card(p2, "Probe Bolt A")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));
    let instant_b = ObjectSpec::card(p2, "Probe Bolt B")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p2));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(instant_a)
        .object(instant_b)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p2);

    let card_a = find_object(&state, "Probe Bolt A");
    let (state, _) =
        process_command(state, cast(p2, card_a)).expect("first cast by p2 should succeed");

    let card_b = find_object(&state, "Probe Bolt B");
    // No intervening PassPriority: p2 responds to their own spell.
    let (state, _) = process_command(state, cast(p2, card_b)).expect(
        "CR 117.3c: p2 should retain priority and be able to cast a second spell without passing",
    );

    assert_eq!(
        state.stack_objects().len(),
        2,
        "both spells should be on the stack"
    );
    assert_eq!(state.turn().priority_holder, Some(p2));
}

// ── P3 — actor can respond to their own activated ability ────────────────────

#[test]
/// CR 117.3c, CR 602.2b — a non-active player who activates an instant-speed
/// activated ability retains priority afterward and can immediately activate a
/// second one, with no intervening pass.
fn test_dp1_actor_can_respond_to_own_activated_ability() {
    let p1 = p(1);
    let p2 = p(2);

    let creature_a = ObjectSpec::creature(p2, "Probe Creature A", 1, 1)
        .with_activated_ability(tap_ability("{T}: Probe effect A"))
        .in_zone(ZoneId::Battlefield);
    let creature_b = ObjectSpec::creature(p2, "Probe Creature B", 1, 1)
        .with_activated_ability(tap_ability("{T}: Probe effect B"))
        .in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(creature_a)
        .object(creature_b)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p2);

    let source_a = find_object(&state, "Probe Creature A");
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: source_a,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("first activation by p2 should succeed");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: p2 retains priority after activating their own ability"
    );

    let source_b = find_object(&state, "Probe Creature B");
    // No intervening PassPriority.
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p2,
            source: source_b,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect(
        "CR 117.3c: p2 should retain priority and be able to activate a second ability without passing",
    );

    assert_eq!(state.stack_objects().len(), 2);
    assert_eq!(state.turn().priority_holder, Some(p2));
}

// ── P4 — cycling by a non-active player retains priority ─────────────────────

#[test]
/// CR 702.29a, CR 602.2b, CR 117.3c — cycling is instant speed with no
/// active-player gate; the cycling player retains priority afterward.
fn test_dp1_non_active_player_cycling_retains_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let cycling_def = CardDefinition {
        card_id: CardId("dp1-cycling-card".to_string()),
        name: "DP1 Cycling Card".to_string(),
        mana_cost: None,
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Cycling {1}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![cycling_def]);

    let cycling_card = ObjectSpec::card(p2, "DP1 Cycling Card")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(CardId("dp1-cycling-card".to_string()))
        .with_keyword(KeywordAbility::Cycling);
    let library_card = ObjectSpec::card(p2, "Library Filler").in_zone(ZoneId::Library(p2));

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(cycling_card)
        .object(library_card)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p2)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p2);

    let card_id = find_object(&state, "DP1 Cycling Card");
    let (state, _) = process_command(
        state,
        Command::CycleCard {
            player: p2,
            card: card_id,
        },
    )
    .expect("CR 702.29a: p2 should be able to cycle during p1's turn");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: the cycling player retains priority"
    );
}

// ── P5 — crewing a vehicle by a non-active player retains priority ───────────

#[test]
/// CR 702.122a, CR 117.3c — crewing has no sorcery-speed gate; the crewing
/// player retains priority afterward.
fn test_dp1_non_active_player_crewing_retains_priority() {
    let p1 = p(1);
    let p2 = p(2);

    let vehicle = vehicle_spec(p2, "Probe Copter", 3, 3, 1);
    let crew_member = ObjectSpec::creature(p2, "Probe Pilot", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(vehicle)
        .object(crew_member)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p2);

    let vehicle_id = find_object(&state, "Probe Copter");
    let pilot_id = find_object(&state, "Probe Pilot");

    let (state, _) = process_command(
        state,
        Command::CrewVehicle {
            player: p2,
            vehicle: vehicle_id,
            crew_creatures: vec![pilot_id],
        },
    )
    .expect("CR 702.122a: p2 should be able to crew during p1's turn");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: the crewing player retains priority"
    );
}

// ── P6 — control probe: active player casting still holds priority ──────────

#[test]
/// CR 117.3c — control probe: when the active player casts a spell they hold
/// priority, they still hold priority afterward. Guards against a mis-targeted
/// edit that writes some other player into `priority_holder`. Green both
/// before and after the fix, by design.
fn test_dp1_active_player_casting_still_holds_priority() {
    let p1 = p(1);

    let instant = ObjectSpec::card(p1, "Probe Bolt")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(instant)
        .build()
        .unwrap();
    // Default priority_holder from the builder is already Some(p1) (the active
    // player), matching "if a player has priority when they cast" (CR 117.3c).

    let card_id = find_object(&state, "Probe Bolt");
    let (state, _) =
        process_command(state, cast(p1, card_id)).expect("active player cast should succeed");

    assert_eq!(state.turn().priority_holder, Some(p1));
}

// ── P7 — PRESERVE: mana ability does not reset players_passed or disturb holder ──

#[test]
/// CR 605.3a/b, CR 117.3b parenthetical — a mana ability does not use the
/// stack and does not disturb the current priority holder. The
/// `players_passed` non-reset is a SEPARATE claim: it pins a known,
/// deliberate CR 117.4 deviation (CR 117.4 requires the all-pass round to
/// restart when "an action" is taken between passes; a mana activation is
/// such an action, but this engine intentionally does not restart the round
/// for it — see OOS-DP1-4). This is the PRESERVE regression pin: it must
/// stay green both before and after the fix. Never weaken this test to make
/// another probe pass.
fn test_dp1_mana_ability_does_not_reset_players_passed() {
    let p1 = p(1);
    let p2 = p(2);

    let forest = ObjectSpec::land(p2, "Probe Forest")
        .with_mana_ability(ManaAbility::tap_for(ManaColor::Green));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(forest)
        .build()
        .unwrap();
    // p1 has already passed; p2 currently holds priority.
    state.turn_mut().players_passed.insert(p1);
    state.turn_mut().priority_holder = Some(p2);

    let land_id = find_object(&state, "Probe Forest");
    let (state, _) = process_command(
        state,
        Command::TapForMana {
            player: p2,
            source: land_id,
            ability_index: 0,
            chosen_color: None,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("tapping for mana should succeed");

    assert!(
        state.turn().players_passed.contains(&p1),
        "known CR 117.4 deviation (see OOS-DP1-4): a mana ability must not reset players_passed"
    );
    assert_eq!(
        state.turn().players_passed.len(),
        1,
        "players_passed must contain exactly the pre-existing entry, nothing added or removed"
    );
    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 605.3a/b: a mana ability does not touch the priority holder"
    );
}

// ── P8 — foretell resets players_passed (CR 117.4) ───────────────────────────

#[test]
/// CR 116.2h, CR 116.3, CR 117.4 — foretelling is a special action; the
/// acting player receives priority afterward (already true by construction,
/// since the priority guard requires the actor to already hold it) and the
/// pass-round restarts because an action was taken between passes.
fn test_dp1_foretell_resets_players_passed() {
    let p1 = p(1);

    let foretell_def = CardDefinition {
        card_id: CardId("dp1-foretell-card".to_string()),
        name: "DP1 Foretell Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Foretell),
            AbilityDefinition::Foretell {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![foretell_def]);

    let card = ObjectSpec::card(p1, "DP1 Foretell Card")
        .with_card_id(CardId("dp1-foretell-card".to_string()))
        .with_keyword(KeywordAbility::Foretell)
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(card)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    // p1 holds priority; simulate p2 and p3 (or a stack object cycle) having
    // already passed earlier in this round.
    state.turn_mut().priority_holder = Some(p1);
    state.turn_mut().players_passed.insert(p(2));
    state.turn_mut().players_passed.insert(p(3));

    let card_id = find_object(&state, "DP1 Foretell Card");
    let (state, _) = process_command(
        state,
        Command::ForetellCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("foretell should succeed");

    assert!(
        state.turn().players_passed.is_empty(),
        "CR 117.4: an action was taken between passes, so the pass-round must restart"
    );
    assert_eq!(
        state.turn().priority_holder,
        Some(p1),
        "CR 116.3: the acting player still holds priority"
    );
}

// ── P9 — special action actor holds priority after turning a permanent face up ──

#[test]
/// CR 116.2b, CR 116.3 — turning a face-down permanent face up is a special
/// action; the player who took it receives priority afterward, and
/// `players_passed` is empty. Since the fix-cycle review (pb-review-DP1.md
/// Finding 1) added an entry priority guard to `handle_turn_face_up`
/// (`rules/engine.rs`), this postcondition now holds *because* the guard
/// already proved `priority_holder == Some(player)` on entry — the tail write
/// is a true identity write, the same shape as the Group-A AP-gated sites.
/// The genuinely discriminating probe for the guard itself is
/// `test_dp1_turn_face_up_rejects_non_priority_holder` below (Finding 2).
fn test_dp1_special_action_actor_holds_priority_after_turn_face_up() {
    let p1 = p(1);
    let p2 = p(2);

    let manifest_def = CardDefinition {
        card_id: CardId("dp1-manifested-creature".to_string()),
        name: "DP1 Manifested Creature".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![manifest_def]);

    let spec = ObjectSpec::card(p2, "DP1 Manifested Creature")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("dp1-manifested-creature".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let face_down_id = find_object(&state, "DP1 Manifested Creature");
    if let Some(obj) = state.objects_mut().get_mut(&face_down_id) {
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Manifest);
    }
    state.turn_mut().priority_holder = Some(p2);

    let (state, _) = process_command(
        state,
        Command::TurnFaceUp {
            player: p2,
            permanent: face_down_id,
            method: TurnFaceUpMethod::ManaCost,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("CR 116.2b: p2 should be able to turn their manifested creature face up");

    assert!(
        state.turn().players_passed.is_empty(),
        "players_passed was already empty and should remain so"
    );
    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 116.3: the player who took the special action receives priority afterward"
    );
}

// ── P10 — fix-cycle: TurnFaceUp guard rejects a non-priority-holder ──────────

#[test]
/// CR 116.2b — turning a face-down permanent face up requires the player to
/// have priority. Fix-cycle addition (pb-review-DP1.md Finding 1/2): p2
/// controls the manifested creature but does NOT hold priority (p1 does);
/// the command must be rejected, not silently grant p2 priority.
///
/// Verified by construction: with the `handle_turn_face_up` entry guard
/// temporarily removed, this probe went RED — `process_command` returned
/// `Ok(..)` instead of `Err`, so `result.is_err()` failed with
/// `assertion failed: result.is_err()`. Restored, it is GREEN.
fn test_dp1_turn_face_up_rejects_non_priority_holder() {
    let p1 = p(1);
    let p2 = p(2);

    let manifest_def = CardDefinition {
        card_id: CardId("dp1-manifested-creature-2".to_string()),
        name: "DP1 Manifested Creature 2".to_string(),
        mana_cost: Some(ManaCost::default()),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![manifest_def]);

    let spec = ObjectSpec::card(p2, "DP1 Manifested Creature 2")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(CardId("dp1-manifested-creature-2".to_string()))
        .with_types(vec![CardType::Creature]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spec)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let face_down_id = find_object(&state, "DP1 Manifested Creature 2");
    if let Some(obj) = state.objects_mut().get_mut(&face_down_id) {
        obj.status.face_down = true;
        obj.face_down_as = Some(FaceDownKind::Manifest);
    }
    // p1 holds priority, NOT p2 (p2 owns/controls the permanent).
    state.turn_mut().priority_holder = Some(p1);

    let result = process_command(
        state,
        Command::TurnFaceUp {
            player: p2,
            permanent: face_down_id,
            method: TurnFaceUpMethod::ManaCost,
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "TurnFaceUp should fail when the actor does not hold priority"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            GameStateError::NotPriorityHolder { .. }
        ),
        "error should be NotPriorityHolder"
    );
}

// ── P11/P12 — fix-cycle: loyalty ability activation grant + guard ───────────

/// Shared planeswalker definition for P11/P12: one `Plus(1)` loyalty ability.
fn dp1_loyalty_pw_def(card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: "DP1 Loyalty Walker".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Planeswalker].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::LoyaltyAbility {
            cost: LoyaltyCost::Plus(1),
            effect: Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            targets: vec![],
        }],
        starting_loyalty: Some(3),
        ..Default::default()
    }
}

#[test]
/// CR 606.1 -> 602.2b -> 601.2i / CR 117.3c — activating a loyalty ability is
/// activating an ability, so the activating player receives priority
/// afterward. Fix-cycle addition (pb-review-DP1.md Finding 2): no probe
/// existed for `handle_activate_loyalty_ability` at all. p2 controls the
/// planeswalker on p1's (the active player's) turn and holds priority; the
/// engine has no "their own turn" gate on loyalty activation (CR 606.3 is
/// under-enforced -- tracked as OOS-DP1-2, out of this PB's scope), so this
/// is a genuine non-active-player flip, not merely an identity write.
///
/// Verified by construction: with the entry guard temporarily removed, this
/// probe stayed green (the pre-existing tail write already granted p2
/// priority) -- so this probe alone does not discriminate the guard. It is
/// kept as the positive/legal-case pin; `test_dp1_loyalty_activation_rejects_non_priority_holder`
/// below is the discriminating guard probe.
fn test_dp1_loyalty_activation_grants_actor_priority() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp1_loyalty_pw_def("dp1-loyalty-pw")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p2, "DP1 Loyalty Walker")
                .with_card_id(CardId("dp1-loyalty-pw".to_string()))
                .with_types(vec![CardType::Planeswalker])
                .with_counter(CounterType::Loyalty, 3)
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p2 (not the active player) holds priority; simulate p1 having passed.
    state.turn_mut().priority_holder = Some(p2);
    state.turn_mut().players_passed.insert(p1);

    let pw_id = find_object(&state, "DP1 Loyalty Walker");
    let (state, _) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p2,
            source: pw_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    )
    .expect("CR 606.1: p2 should be able to activate their own planeswalker's loyalty ability");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: p2 retains priority after activating their loyalty ability"
    );
    assert!(
        state.turn().players_passed.is_empty(),
        "CR 117.4: an action was taken between passes, so the pass-round must restart"
    );
}

#[test]
/// CR 606.3 — activating a loyalty ability requires the player to have
/// priority. Fix-cycle addition (pb-review-DP1.md Finding 1/2): p1 controls
/// the planeswalker but does NOT hold priority (p2 does); the command must be
/// rejected outright, not silently grant p1 priority.
///
/// Verified by construction: with the `handle_activate_loyalty_ability` entry
/// guard temporarily removed, this probe went RED — `process_command`
/// returned `Ok(..)` and the pre-existing tail write handed p1 priority it
/// never held, so `result.is_err()` failed. Restored, it is GREEN.
fn test_dp1_loyalty_activation_rejects_non_priority_holder() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp1_loyalty_pw_def("dp1-loyalty-pw-2")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "DP1 Loyalty Walker")
                .with_card_id(CardId("dp1-loyalty-pw-2".to_string()))
                .with_types(vec![CardType::Planeswalker])
                .with_counter(CounterType::Loyalty, 3)
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // p2 holds priority, NOT p1 (p1 owns/controls the planeswalker).
    state.turn_mut().priority_holder = Some(p2);

    let pw_id = find_object(&state, "DP1 Loyalty Walker");
    let result = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: pw_id,
            ability_index: 0,
            targets: vec![],
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "ActivateLoyaltyAbility should fail when the actor does not hold priority"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            GameStateError::NotPriorityHolder { .. }
        ),
        "error should be NotPriorityHolder"
    );
}

// ── P13/P14 — fix-cycle: level-up-a-Class grant + guard ─────────────────────

/// Shared Class definition for P13/P14: one level-2 `ClassLevel` ability.
fn dp1_class_def(card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: "DP1 Test Class".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            subtypes: [SubType("Class".to_string())].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::ClassLevel {
            level: 2,
            cost: ManaCost {
                generic: 1,
                ..Default::default()
            },
            abilities: vec![],
        }],
        ..Default::default()
    }
}

#[test]
/// CR 716.2a -> 602.2b -> 601.2i / CR 117.3c — leveling up a Class is
/// activating an ability, so the activating player receives priority
/// afterward. Fix-cycle addition (pb-review-DP1.md Finding 2): no probe
/// existed for `handle_level_up_class` at all. p2 controls the Class on p1's
/// (the active player's) turn and holds priority; like loyalty abilities,
/// `handle_level_up_class` has no "their own turn" gate (OOS-DP1-2), so this
/// is a genuine non-active-player flip.
fn test_dp1_level_up_class_grants_actor_priority() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp1_class_def("dp1-class")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p2, "DP1 Test Class")
                .with_card_id(CardId("dp1-class".to_string()))
                .with_types(vec![CardType::Enchantment])
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let class_id = find_object(&state, "DP1 Test Class");
    if let Some(obj) = state.objects_mut().get_mut(&class_id) {
        obj.class_level = 1;
    }
    if let Some(ps) = state.players_mut().get_mut(&p2) {
        ps.mana_pool.add(ManaColor::Colorless, 1);
    }
    // p2 (not the active player) holds priority; simulate p1 having passed.
    state.turn_mut().priority_holder = Some(p2);
    state.turn_mut().players_passed.insert(p1);

    let (state, _) = process_command(
        state,
        Command::LevelUpClass {
            player: p2,
            source: class_id,
            target_level: 2,
        },
    )
    .expect("CR 716.2a: p2 should be able to level up their own Class");

    assert_eq!(
        state.turn().priority_holder,
        Some(p2),
        "CR 117.3c: p2 retains priority after leveling up their Class"
    );
    assert!(
        state.turn().players_passed.is_empty(),
        "CR 117.4: an action was taken between passes, so the pass-round must restart"
    );
}

#[test]
/// CR 716.2a — leveling up a Class requires the player to have priority.
/// Fix-cycle addition (pb-review-DP1.md Finding 1/2): p1 controls the Class
/// but does NOT hold priority (p2 does); the command must be rejected.
///
/// Verified by construction: with the `handle_level_up_class` entry guard
/// temporarily removed, this probe went RED — `process_command` returned
/// `Ok(..)` and the pre-existing tail write handed p1 priority it never
/// held, so `result.is_err()` failed. Restored, it is GREEN.
fn test_dp1_level_up_class_rejects_non_priority_holder() {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![dp1_class_def("dp1-class-2")]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "DP1 Test Class")
                .with_card_id(CardId("dp1-class-2".to_string()))
                .with_types(vec![CardType::Enchantment])
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let class_id = find_object(&state, "DP1 Test Class");
    if let Some(obj) = state.objects_mut().get_mut(&class_id) {
        obj.class_level = 1;
    }
    if let Some(ps) = state.players_mut().get_mut(&p1) {
        ps.mana_pool.add(ManaColor::Colorless, 1);
    }
    // p2 holds priority, NOT p1 (p1 owns/controls the Class).
    state.turn_mut().priority_holder = Some(p2);

    let result = process_command(
        state,
        Command::LevelUpClass {
            player: p1,
            source: class_id,
            target_level: 2,
        },
    );

    assert!(
        result.is_err(),
        "LevelUpClass should fail when the actor does not hold priority"
    );
    assert!(
        matches!(
            result.unwrap_err(),
            GameStateError::NotPriorityHolder { .. }
        ),
        "error should be NotPriorityHolder"
    );
}

// ── P15/P16/P17 — fix-cycle: D-b `players_passed` reset coverage ────────────
//
// pb-review-DP1.md LOW 7: D-b coverage was 1-of-4 (foretell only). These three
// probes close plot / suspend / bring_companion.

#[test]
/// CR 116.2k, CR 116.3, CR 117.4 — plotting a card is a special action; the
/// pass-round restarts because an action was taken between passes.
fn test_dp1_plot_resets_players_passed() {
    let p1 = p(1);

    let plot_def = CardDefinition {
        card_id: CardId("dp1-plot-card".to_string()),
        name: "DP1 Plot Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Plot),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Plot,
                details: None,
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![plot_def]);

    let card = ObjectSpec::card(p1, "DP1 Plot Card")
        .with_card_id(CardId("dp1-plot-card".to_string()))
        .with_keyword(KeywordAbility::Plot)
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(card)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);
    state.turn_mut().players_passed.insert(p(2));
    state.turn_mut().players_passed.insert(p(3));

    let card_id = find_object(&state, "DP1 Plot Card");
    let (state, _) = process_command(
        state,
        Command::PlotCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("plot should succeed");

    assert!(
        state.turn().players_passed.is_empty(),
        "CR 117.4: an action was taken between passes, so the pass-round must restart"
    );
    assert_eq!(state.turn().priority_holder, Some(p1));
}

#[test]
/// CR 116.2f, CR 116.3, CR 117.4 — suspending a card is a special action; the
/// pass-round restarts because an action was taken between passes.
fn test_dp1_suspend_resets_players_passed() {
    let p1 = p(1);

    let suspend_def = CardDefinition {
        card_id: CardId("dp1-suspend-card".to_string()),
        name: "DP1 Suspend Card".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: mtg_engine::TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Suspend),
            AbilityDefinition::Suspend {
                cost: ManaCost {
                    generic: 1,
                    ..Default::default()
                },
                time_counters: 2,
            },
        ],
        ..Default::default()
    };
    let registry = CardRegistry::new(vec![suspend_def]);

    let card = ObjectSpec::card(p1, "DP1 Suspend Card")
        .with_card_id(CardId("dp1-suspend-card".to_string()))
        .with_keyword(KeywordAbility::Suspend)
        .with_types(vec![CardType::Sorcery])
        .in_zone(ZoneId::Hand(p1));

    let mut state = GameStateBuilder::four_player()
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(card)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p1);
    state.turn_mut().players_passed.insert(p(2));
    state.turn_mut().players_passed.insert(p(3));

    let card_id = find_object(&state, "DP1 Suspend Card");
    let (state, _) = process_command(
        state,
        Command::SuspendCard {
            player: p1,
            card: card_id,
        },
    )
    .expect("suspend should succeed");

    assert!(
        state.turn().players_passed.is_empty(),
        "CR 117.4: an action was taken between passes, so the pass-round must restart"
    );
    assert_eq!(state.turn().priority_holder, Some(p1));
}

#[test]
/// CR 702.139a, CR 116.3, CR 117.4 — bringing a companion to hand is a
/// special action; the pass-round restarts because an action was taken
/// between passes.
fn test_dp1_bring_companion_resets_players_passed() {
    let p1 = p(1);

    let companion_card = ObjectSpec::card(p1, "DP1 Companion")
        .with_card_id(CardId("dp1-companion".to_string()))
        .in_zone(ZoneId::Command(p1));

    let mut state = GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(companion_card)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state.players_mut().get_mut(&p1).unwrap().companion = Some(CardId("dp1-companion".to_string()));
    state.turn_mut().priority_holder = Some(p1);
    state.turn_mut().players_passed.insert(p(2));
    state.turn_mut().players_passed.insert(p(3));

    let (state, _) = process_command(state, Command::BringCompanion { player: p1 })
        .expect("companion should succeed");

    assert!(
        state.turn().players_passed.is_empty(),
        "CR 117.4: an action was taken between passes, so the pass-round must restart"
    );
}
