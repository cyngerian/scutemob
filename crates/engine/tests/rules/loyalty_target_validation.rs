//! Loyalty-ability target validation tests — PB-LS6, Issue L01.
//!
//! Verifies that `handle_activate_loyalty_ability` validates declared targets against
//! the ability's `TargetRequirement`s BEFORE paying the loyalty cost (CR 601.2c / CR 606.4).
//!
//! Prior to PB-LS6, the loyalty handler accepted any target regardless of type — a player
//! could activate Sorin -6 targeting a Land object with no validation error, and the
//! loyalty cost would be consumed. After PB-LS6, `validate_targets_with_source` is called
//! before the cost-payment mutation.
//!
//! CR refs:
//!   CR 601.2c — target announcement/validation
//!   CR 606.3  — loyalty ability timing / once-per-turn restriction
//!   CR 606.4  — loyalty cost is paid when activated
//!   CR 606.6  — insufficient loyalty counters → invalid activation

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    CounterType, Effect, EffectAmount, GameStateBuilder, GameStateError, LoyaltyCost, ManaCost,
    ObjectSpec, PlayerId, PlayerTarget, Step, Target, TargetFilter, TargetRequirement, TypeLine,
    ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Minimal Sorin-style planeswalker definition:
/// - +1: gain 1 life (no target)
/// - -2: destroy target creature or planeswalker (UpToN{1}, Creature|Planeswalker)
/// - -6: destroy up to 3 target creatures or planeswalkers + reanimate (UpToN{3})
fn sorin_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-sorin".to_string()),
        name: "Test Sorin".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            black: 1,
            ..Default::default()
        }),
        types: TypeLine {
            supertypes: imbl::OrdSet::new(),
            card_types: imbl::ordset![CardType::Planeswalker],
            subtypes: imbl::OrdSet::new(),
        },
        oracle_text: String::new(),
        starting_loyalty: Some(6),
        abilities: vec![
            // +1: gain 1 life (no target)
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                targets: vec![],
            },
            // -2: destroy up to 1 target creature or planeswalker
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(2),
                effect: Effect::DestroyPermanent {
                    target: mtg_engine::CardEffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                        ..Default::default()
                    })),
                }],
            },
            // -6: destroy up to 3 target creatures/planeswalkers + reanimate
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                effect: Effect::DestroyAndReanimate {
                    targets: vec![
                        mtg_engine::CardEffectTarget::DeclaredTarget { index: 0 },
                        mtg_engine::CardEffectTarget::DeclaredTarget { index: 1 },
                        mtg_engine::CardEffectTarget::DeclaredTarget { index: 2 },
                    ],
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::UpToN {
                    count: 3,
                    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                        ..Default::default()
                    })),
                }],
            },
        ],
        ..Default::default()
    }
}

/// Build a test state with the Sorin-like planeswalker and a creature + a land for P1.
/// P2 is present to satisfy multiplayer requirements.
fn build_state_with_targets() -> (
    mtg_engine::GameState,
    mtg_engine::ObjectId, // sorin id
    mtg_engine::ObjectId, // creature id
    mtg_engine::ObjectId, // land id
    PlayerId,
) {
    let p1 = p(1);
    let p2 = p(2);
    let registry = CardRegistry::new(vec![sorin_def()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(
            ObjectSpec::card(p1, "Test Sorin")
                .with_card_id(CardId("test-sorin".to_string()))
                .with_types(vec![CardType::Planeswalker])
                .with_counter(CounterType::Loyalty, 6)
                .in_zone(ZoneId::Battlefield),
        )
        .object(ObjectSpec::creature(p2, "Target Creature", 2, 2))
        .object(
            ObjectSpec::card(p2, "Target Land")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let sorin_id = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Test Sorin")
        .unwrap()
        .id;
    let creature_id = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Target Creature")
        .unwrap()
        .id;
    let land_id = state
        .objects()
        .values()
        .find(|o| o.characteristics.name == "Target Land")
        .unwrap()
        .id;

    (state, sorin_id, creature_id, land_id, p1)
}

// ── Test 1: Illegal target type is rejected ─────────────────────────────────────

/// CR 601.2c — A loyalty ability that targets "creatures and/or planeswalkers" must
/// reject a Land as the declared target.
#[test]
fn test_l01_loyalty_ability_rejects_illegal_target_type() {
    let (state, sorin_id, _creature_id, land_id, p1) = build_state_with_targets();

    // Activate -2 (index 1) targeting a Land — Land is not a creature or planeswalker.
    let result = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sorin_id,
            ability_index: 1, // -2: destroy target creature/planeswalker
            targets: vec![Target::Object(land_id)],
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "CR 601.2c: activating a loyalty ability with an illegal target type must fail"
    );
    match result.unwrap_err() {
        GameStateError::InvalidTarget(_) | GameStateError::InvalidCommand(_) => {}
        e => panic!("expected InvalidTarget or InvalidCommand, got {:?}", e),
    }
}

// ── Test 2: Legal target is accepted ──────────────────────────────────────────

/// CR 601.2c / CR 606.4 — Activating a loyalty ability with a legal target succeeds;
/// loyalty cost is paid and the ability goes on the stack.
#[test]
fn test_l01_loyalty_ability_accepts_legal_target() {
    let (state, sorin_id, creature_id, _land_id, p1) = build_state_with_targets();

    let result = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sorin_id,
            ability_index: 1, // -2: destroy target creature/planeswalker
            targets: vec![Target::Object(creature_id)],
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 601.2c: a legal target must be accepted; got {:?}",
        result.err()
    );
    let (state2, _) = result.unwrap();
    // Loyalty decreased by 2 (from 6 → 4).
    let sorin = state2.objects().get(&sorin_id).unwrap();
    assert_eq!(
        sorin
            .counters
            .get(&CounterType::Loyalty)
            .copied()
            .unwrap_or(0),
        4,
        "CR 606.4: loyalty cost (-2) must be paid on successful activation"
    );
    // Ability is on the stack.
    assert_eq!(
        state2.stack_objects().len(),
        1,
        "ability should be on the stack after activation"
    );
}

// ── Test 3: Zero declared targets is legal for UpToN ─────────────────────────

/// CR 601.2c — "Up to N" requirements allow declaring zero targets. Activating with
/// zero targets must succeed.
#[test]
fn test_l01_loyalty_ability_zero_targets_legal() {
    let (state, sorin_id, _creature_id, _land_id, p1) = build_state_with_targets();

    // Activate -2 (UpToN{1}) with 0 declared targets — min is 0, so this is legal.
    let result = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sorin_id,
            ability_index: 1, // -2 UpToN{1}
            targets: vec![],
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 601.2c: UpToN with 0 declared targets must be legal (min targets = 0)"
    );
}

// ── Test 4: Loyalty NOT paid on rejected activation ───────────────────────────

/// CR 606.4 / CR 606.3 — the loyalty cost is paid when the ability is activated
/// SUCCESSFULLY, exactly once per turn, and an illegal target is refused instead.
///
/// **`OOS-DX21-7` repair (PB-DX57), and the residual it could not close — read this
/// before "strengthening" the test.**
///
/// This test used to snapshot `initial_loyalty` and `initially_activated` from the
/// fixture, MOVE `state` into `process_command`, and then assert `initial_loyalty == 6`
/// and `!initially_activated` — two `let` bindings compared against the literals they
/// had just been read from, with nothing between the snapshot and the assertion able to
/// write either. Its own comment ("we cannot inspect the state after a failed command")
/// stated the mechanism and asserted anyway.
///
/// **The direct-handler repair is UNBUILDABLE here, and no engine line was taken to
/// make it buildable.** `Command::ActivateLoyaltyAbility` dispatches to
/// `fn handle_activate_loyalty_ability` in `crates/engine/src/rules/engine.rs` — a
/// PRIVATE `fn`, re-exported nowhere — and an integration test can only name `pub`
/// items. Nine other command variants are in the same position (`handle_pass_priority`,
/// `handle_concede`, `handle_transform`, `handle_turn_face_up`, `handle_level_up_class`,
/// `handle_activate_craft`, `handle_pay_echo`, `handle_pay_recover`,
/// `handle_pay_cumulative_upkeep`).
///
/// **STATED RESIDUAL.** What this test now establishes is what the ACCEPTED path
/// produces — that an illegal target is refused, that a legal activation pays exactly
/// the printed cost and sets the once-per-turn marker, and that a second activation is
/// then refused. What it does NOT and CANNOT establish from an integration test is the
/// property the old assertions pretended to check: that
/// `handle_activate_loyalty_ability` performs **no mutation before it validates**. That
/// remains unbuildable for this command and for the nine listed above until either the
/// handler becomes `pub` or `process_command` grows a `&mut GameState` sibling
/// (`process_command_mut`), which is the only repair that makes the property falsifiable
/// for all 45 `Command` variants at once. Do not re-add a snapshot-versus-literal
/// assertion in its place: it reads as coverage and measures nothing.
#[test]
fn test_l01_loyalty_cost_not_paid_on_rejected_activation() {
    let (state, sorin_id, creature_id, land_id, p1) = build_state_with_targets();

    // Non-vacuity floor: the fixture really is the pre-activation state the assertions
    // below are differences FROM.
    assert_eq!(
        state
            .objects()
            .get(&sorin_id)
            .and_then(|o| o.counters.get(&CounterType::Loyalty).copied())
            .unwrap_or(0),
        6,
        "the fixture starts Sorin at 6 loyalty"
    );
    assert!(
        !state
            .objects()
            .get(&sorin_id)
            .map(|o| o.loyalty_ability_activated_this_turn)
            .unwrap_or(true),
        "and with the CR 606.3 once-per-turn marker clear"
    );

    // (1) The illegal target is refused (CR 601.2c).
    let err = process_command(
        state.clone(),
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sorin_id,
            ability_index: 1, // -2: requires creature/planeswalker
            targets: vec![Target::Object(land_id)], // Land — illegal
            x_value: None,
        },
    )
    .expect_err("illegal target must cause activation failure");
    assert!(
        matches!(
            err,
            GameStateError::InvalidTarget(_) | GameStateError::InvalidCommand(_)
        ),
        "expected InvalidTarget or InvalidCommand, got {err:?}"
    );

    // (2) The observable the ACCEPTED path produces: exactly -2, and the marker set.
    // This is what CR 606.4 says, read off a state the engine really returned.
    let (after, _) = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sorin_id,
            ability_index: 1,
            targets: vec![Target::Object(creature_id)],
            x_value: None,
        },
    )
    .expect("a legal target must be accepted");
    let sorin = after
        .objects()
        .get(&sorin_id)
        .expect("Sorin is still there");
    assert_eq!(
        sorin
            .counters
            .get(&CounterType::Loyalty)
            .copied()
            .unwrap_or(0),
        4,
        "CR 606.4: an accepted -2 pays exactly 2 loyalty, 6 -> 4"
    );
    assert!(
        sorin.loyalty_ability_activated_this_turn,
        "CR 606.3: and sets the once-per-turn marker"
    );

    // (3) And that marker bites: a second activation on THAT returned state is refused.
    let err2 = process_command(
        after,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sorin_id,
            ability_index: 0, // +1: no target, so only CR 606.3 can refuse it
            targets: vec![],
            x_value: None,
        },
    )
    .expect_err("CR 606.3: only one loyalty ability per turn");
    assert!(
        matches!(err2, GameStateError::InvalidCommand(_)),
        "expected the CR 606.3 once-per-turn refusal, got {err2:?}"
    );
}

// ── Test 5: No-target ability is unaffected by L01 ───────────────────────────

/// CR 606.3 — Abilities with empty `targets: vec![]` skip validation entirely (the
/// `if !ability_targets.is_empty()` guard). Sorin +1 must still activate correctly.
#[test]
fn test_l01_no_target_ability_unaffected() {
    let (state, sorin_id, _creature_id, _land_id, p1) = build_state_with_targets();

    let result = process_command(
        state,
        Command::ActivateLoyaltyAbility {
            player: p1,
            source: sorin_id,
            ability_index: 0, // +1: no target, gains life
            targets: vec![],
            x_value: None,
        },
    );

    assert!(
        result.is_ok(),
        "CR 606.3: a no-target loyalty ability must still activate correctly"
    );
    let (state2, _) = result.unwrap();
    let sorin = state2.objects().get(&sorin_id).unwrap();
    assert_eq!(
        sorin
            .counters
            .get(&CounterType::Loyalty)
            .copied()
            .unwrap_or(0),
        7, // 6 + 1
        "CR 606.4: +1 loyalty ability should increase loyalty from 6 to 7"
    );
}

// ── HASH_SCHEMA_VERSION sentinel ──────────────────────────────────────────────

/// BASELINE-LKI-01 bump: HASH_SCHEMA_VERSION = 27 (GameEvent::CreatureDied.pre_death_characteristics,
/// CR 603.10a / CR 613.1d LKI snapshot for filtered death triggers).
#[test]
fn test_pb_ls6_hash_schema_version_is_26() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 85u8,
        "HASH_SCHEMA_VERSION drifted without this sentinel being updated. Bump this assertion and the state/hash.rs history block together; the authoritative check is the SR-17 machine gate in tests/core/hash_schema.rs."
    );
}
