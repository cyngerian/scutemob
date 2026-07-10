//! Tests for PB-AC8: static restrictions, win-cons, no-max-hand cleanup fix.
//!
//! Covers three genuinely-new engine primitives plus one bug fix:
//!
//! - `GameRestriction::CantAttackOwner` (CR 508.1c) — self-referential restriction,
//!   keyed on the attacker's OWNER (not controller). Interacts with
//!   `KeywordAbility::MustAttackEachCombat` per CR 508.1d: a requirement is obeyed
//!   only to the extent it doesn't violate a restriction.
//! - `GameRestriction::CantBeSacrificed` (CR 701.21a) — full dispatch-chain wiring
//!   through a single choke-point helper (`crate::effects::object_cant_be_sacrificed`).
//! - `Effect::WinGame` (CR 104.1 / 104.2b / 104.3f) — no `condition` field by design;
//!   gating comes from the ability's own `intervening_if` or `Effect::Conditional`.
//!   Marks every other still-active player `has_lost`; does NOT add a `has_won` field
//!   and does NOT touch `sba.rs` (CR 704.5: winning-by-effect is not an SBA).
//! - Bug fix: cleanup discard's `has_no_max` scan now uses layer-resolved
//!   characteristics (`calculate_characteristics`), so a layer-granted
//!   `NoMaxHandSize` (e.g. Wrenn and Seven's emblem proxy) is no longer invisible.
//!
//! Hash: `HASH_SCHEMA_VERSION` bumped 34 -> 35 (new `GameRestriction::CantAttackOwner`
//! disc 9, `GameRestriction::CantBeSacrificed` disc 10, `Effect::WinGame` disc 90,
//! `LossReason::OpponentWonGame` disc 5). No new mutable GameState/PlayerState/
//! GameObject fields this batch (WinGame reuses `PlayerState.has_lost`).

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Condition, Effect, EffectAmount, PlayerTarget, TargetFilter,
    TriggerCondition,
};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::stubs::{ActiveRestriction, PendingTrigger, PendingTriggerKind};
use mtg_engine::state::test_util;
use mtg_engine::state::{ActivatedAbility, ActivationCost, SacrificeFilter};
use mtg_engine::{
    process_command, AttackTarget, CardId, CardType, Command, GameEvent, GameRestriction,
    GameState, GameStateBuilder, KeywordAbility, ManaCost, ObjectId, ObjectSpec, PlayerId, Step,
    TypeLine, ZoneId, HASH_SCHEMA_VERSION,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_by_name(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

/// Run an effect directly (bypasses casting/resolution machinery).
fn run_effect(
    mut state: GameState,
    controller: PlayerId,
    effect: Effect,
) -> (GameState, Vec<GameEvent>) {
    let source = ObjectId(0);
    let mut ctx = EffectContext::new(controller, source, vec![]);
    let events = execute_effect(&mut state, &effect, &mut ctx);
    (state, events)
}

fn add_restriction(
    state: &mut GameState,
    source: ObjectId,
    controller: PlayerId,
    restriction: GameRestriction,
) {
    state.restrictions_mut().push_back(ActiveRestriction {
        source,
        controller,
        restriction,
    });
}

fn declare_cmd(player: PlayerId, attackers: Vec<(ObjectId, AttackTarget)>) -> Command {
    Command::DeclareAttackers {
        player,
        attackers,
        enlist_choices: vec![],
        exert_choices: vec![],
    }
}

fn cast_cmd(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell {
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
    }
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        events.extend(ev);
    }
    (current, events)
}

/// Build a simple sorcery CardDefinition whose spell effect is the given `Effect`.
fn sorcery_def(name: &str, card_id: &str, effect: Effect) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Sorcery].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("Test sorcery: {}.", name),
        abilities: vec![AbilityDefinition::Spell {
            effect,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── HASH_SCHEMA_VERSION sentinel ──────────────────────────────────────────────

/// HASH_SCHEMA_VERSION live sentinel — fails if the schema version drifts without
/// this test being updated. PB-AC8 bumped 34 -> 35.
#[test]
fn test_pb_ac8_hash_schema_version_live_sentinel() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 37u8,
        "PB-AC8 bumped HASH_SCHEMA_VERSION 34->35 (GameRestriction::CantAttackOwner disc 9, \
         GameRestriction::CantBeSacrificed disc 10, Effect::WinGame disc 90, \
         LossReason::OpponentWonGame disc 5). If you bumped again, update this test."
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Effect::WinGame (CR 104.1 / 104.2b / 104.3f)
// ═══════════════════════════════════════════════════════════════════════════

/// CR 104.1 / 104.2b — 1v1: the controller wins, the sole opponent loses.
#[test]
fn test_wingame_1v1_controller_wins_opponent_loses() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();

    let (state, events) = run_effect(state, p(1), Effect::WinGame);

    assert!(
        state.players().get(&p(2)).unwrap().has_lost,
        "opponent must lose when the controller wins the game (CR 104.1)"
    );
    assert!(
        !state.players().get(&p(1)).unwrap().has_lost,
        "the winning controller must not be marked has_lost"
    );
    assert!(events.iter().any(|e| matches!(
        e,
        GameEvent::PlayerLost {
            player,
            reason: mtg_engine::LossReason::OpponentWonGame
        } if *player == p(2)
    )));
}

/// CR 104.1 (limited-range-of-influence option absent — Commander does not use it,
/// CR 801) — architecture invariant #5 (multiplayer-first): in a 4-player Commander
/// pod, "you win the game" eliminates ALL three opponents and ends the game
/// immediately, not just "opponents in range" (that's the 104.3h/801.14 variant,
/// which does not apply here). MANDATORY test.
#[test]
fn test_wingame_4player_all_three_opponents_lose() {
    let registry = mtg_engine::CardRegistry::new(vec![sorcery_def(
        "Win The Game",
        "win-the-game",
        Effect::WinGame,
    )]);

    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::card(p(1), "Win The Game")
                .in_zone(ZoneId::Hand(p(1)))
                .with_card_id(CardId("win-the-game".to_string()))
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..Default::default()
                }),
        )
        .with_registry(registry)
        .at_step(Step::PreCombatMain)
        .active_player(p(1))
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 1);
    state.turn_mut().priority_holder = Some(p(1));

    let card = find_by_name(&state, "Win The Game");
    let (state, cast_events) =
        process_command(state, cast_cmd(p(1), card)).expect("CastSpell should succeed");

    // Resolve the spell: P1 passes, P2, P3, P4 pass (all four pass in succession
    // resolves the top of the stack, CR 608.1).
    let (state, resolve_events) = pass_all(state, &[p(1), p(2), p(3), p(4)]);
    let mut all_events = cast_events;
    all_events.extend(resolve_events);

    for opp in [p(2), p(3), p(4)] {
        assert!(
            state.players().get(&opp).unwrap().has_lost,
            "player {:?} should have lost when P1 won the game (CR 104.1)",
            opp
        );
    }
    assert!(
        !state.players().get(&p(1)).unwrap().has_lost,
        "the winning controller P1 must not be marked has_lost"
    );
    let active = state.active_players();
    assert_eq!(
        active,
        vec![p(1)],
        "exactly one active player (P1) should remain"
    );
    assert!(
        all_events.iter().any(|e| matches!(
            e,
            GameEvent::GameOver { winner: Some(w) } if *w == p(1)
        )),
        "GameOver {{ winner: Some(P1) }} must be emitted; events: {:?}",
        all_events
    );
}

/// CR 104.3f — "If a player would both win and lose the game simultaneously, that
/// player loses." WinGame must no-op if the controller has already lost.
#[test]
fn test_wingame_controller_already_lost_is_noop() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();
    state.players_mut().get_mut(&p(1)).unwrap().has_lost = true;

    let (state, events) = run_effect(state, p(1), Effect::WinGame);

    assert!(
        !state.players().get(&p(2)).unwrap().has_lost,
        "opponent must NOT lose when the (already-lost) controller 'wins' (CR 104.3f)"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::PlayerLost { .. })),
        "no PlayerLost events should be emitted; events: {:?}",
        events
    );
}

/// Hazard F mutation-verification: the public state hash must differ before and
/// after a WinGame resolution eliminates an opponent (`has_lost` mutation).
#[test]
fn test_wingame_hashes_change_on_elimination() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .build()
        .unwrap();

    let hash_before = state.public_state_hash();
    let (state, _events) = run_effect(state, p(1), Effect::WinGame);
    let hash_after = state.public_state_hash();

    assert_ne!(
        hash_before, hash_after,
        "state hash must change when WinGame eliminates an opponent"
    );
}

/// Validates the no-`condition`-field `Effect::WinGame` design: gating comes from
/// the triggered ability's own `intervening_if` (CR 603.4), re-checked at
/// resolution. False below the threshold (ability resolves with no effect); wins
/// once the threshold is met. Abstracted from Hellkite Tyrant's mechanism
/// ("at the beginning of your upkeep, if you control 20+ artifacts, you win") with
/// a smaller threshold (2) for test economy.
#[test]
fn test_wingame_via_intervening_if_upkeep_trigger() {
    let win_con = CardDefinition {
        card_id: CardId("test-win-con".to_string()),
        name: "Test Win Con".to_string(),
        mana_cost: None,
        types: TypeLine {
            card_types: [CardType::Artifact].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "At the beginning of your upkeep, if you control two or more \
                      artifacts, you win the game."
            .to_string(),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
            effect: Effect::WinGame,
            intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                count: 2,
                filter: TargetFilter {
                    has_card_type: Some(CardType::Artifact),
                    ..Default::default()
                },
            }),
            targets: vec![],
            modes: None,
            trigger_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };
    let registry = mtg_engine::CardRegistry::new(vec![win_con]);

    // ── Below threshold: only 1 artifact (the win-con itself). ──
    let mut state_below = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Test Win Con")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-win-con".to_string()))
                .with_types(vec![CardType::Artifact]),
        )
        .with_registry(registry.clone())
        .build()
        .unwrap();

    let win_con_id = find_by_name(&state_below, "Test Win Con");
    state_below
        .pending_triggers_mut()
        .push_back(PendingTrigger::blank(
            win_con_id,
            p(1),
            PendingTriggerKind::Normal,
        ));
    let events_below = mtg_engine::rules::abilities::flush_pending_triggers(&mut state_below);
    let mut all_events_below = events_below;
    // Resolve the stack (all pass).
    let (state_below, more_events) = pass_all(state_below, &[p(1), p(2)]);
    all_events_below.extend(more_events);

    assert!(
        !state_below.players().get(&p(2)).unwrap().has_lost,
        "below threshold (1 artifact < 2): WinGame must NOT resolve (CR 603.4 \
         intervening-if false at resolution)"
    );

    // ── At threshold: 2 artifacts. ──
    let mut state_at = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(
            ObjectSpec::card(p(1), "Test Win Con")
                .in_zone(ZoneId::Battlefield)
                .with_card_id(CardId("test-win-con".to_string()))
                .with_types(vec![CardType::Artifact]),
        )
        .object(ObjectSpec::artifact(p(1), "Second Artifact").in_zone(ZoneId::Battlefield))
        .with_registry(registry)
        .build()
        .unwrap();

    let win_con_id = find_by_name(&state_at, "Test Win Con");
    state_at
        .pending_triggers_mut()
        .push_back(PendingTrigger::blank(
            win_con_id,
            p(1),
            PendingTriggerKind::Normal,
        ));
    let events_at = mtg_engine::rules::abilities::flush_pending_triggers(&mut state_at);
    let mut all_events_at = events_at;
    let (state_at, more_events) = pass_all(state_at, &[p(1), p(2)]);
    all_events_at.extend(more_events);

    assert!(
        state_at.players().get(&p(2)).unwrap().has_lost,
        "at threshold (2 artifacts >= 2): WinGame must resolve and eliminate the \
         opponent; events: {:?}",
        all_events_at
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// GameRestriction::CantBeSacrificed (CR 701.21a) — full dispatch chain
// ═══════════════════════════════════════════════════════════════════════════

/// CR 701.21a — edict effect (`Effect::SacrificePermanents`) skips a protected
/// permanent: when the only permanent controlled by the player is protected,
/// nothing is sacrificed.
#[test]
fn test_cant_be_sacrificed_edict_skips_protected() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Protected Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let bear = find_by_name(&state, "Protected Bear");
    add_restriction(&mut state, bear, p(1), GameRestriction::CantBeSacrificed);

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::SacrificePermanents {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: None,
        },
    );

    assert!(
        on_battlefield(&state, "Protected Bear"),
        "a can't-be-sacrificed permanent must survive an edict when it's the only target"
    );
    assert!(
        !in_graveyard(&state, "Protected Bear", p(1)),
        "protected permanent must not be in the graveyard"
    );
}

/// CR 701.21a + CR 109.1 — when choosing which permanent to sacrifice among
/// several, a protected permanent is excluded from the eligible set; the
/// unprotected permanent is chosen instead (validates `eligible_sacrifice_targets`
/// filtering without depending on it directly — black-box).
#[test]
fn test_cant_be_sacrificed_choice_excludes_from_eligible() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Protected Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(1), "Normal Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let bear = find_by_name(&state, "Protected Bear");
    add_restriction(&mut state, bear, p(1), GameRestriction::CantBeSacrificed);

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::SacrificePermanents {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: None,
        },
    );

    assert!(
        on_battlefield(&state, "Protected Bear"),
        "protected permanent must remain on the battlefield"
    );
    assert!(
        in_graveyard(&state, "Normal Bear", p(1)),
        "the unprotected permanent must be sacrificed instead"
    );
}

/// CR 602.2c + CR 701.21a — a "Sacrifice this:" activated ability cannot be
/// activated when the source can't be sacrificed (activation cost can't be paid).
#[test]
fn test_cant_be_sacrificed_activation_cost_cannot_pay() {
    let sac_creature = ObjectSpec::creature(p(1), "Suicide Bomber", 1, 1).with_activated_ability(
        ActivatedAbility {
            cost: ActivationCost {
                sacrifice_self: true,
                ..Default::default()
            },
            description: "Sacrifice this: deal 1 damage to each opponent".to_string(),
            effect: Some(Effect::DealDamage {
                target: mtg_engine::CardEffectTarget::EachOpponent,
                amount: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        },
    );

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(sac_creature)
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let creature_id = find_by_name(&state, "Suicide Bomber");
    add_restriction(
        &mut state,
        creature_id,
        p(1),
        GameRestriction::CantBeSacrificed,
    );

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: creature_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "a can't-be-sacrificed permanent must not be able to pay a sacrifice-self cost"
    );
}

/// CR 602.2 + CR 701.21a — a "Sacrifice a creature:" cost cannot target a
/// can't-be-sacrificed creature.
#[test]
fn test_cant_be_sacrificed_activation_cost_sacrifice_filter_cannot_pay() {
    let source =
        ObjectSpec::artifact(p(1), "Blood Altar").with_activated_ability(ActivatedAbility {
            cost: ActivationCost {
                sacrifice_filter: Some(SacrificeFilter::Creature),
                ..Default::default()
            },
            description: "Sacrifice a creature: draw a card".to_string(),
            effect: Some(Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            }),
            sorcery_speed: false,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        });

    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(source)
        .object(ObjectSpec::creature(p(1), "Protected Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(1));

    let altar = find_by_name(&state, "Blood Altar");
    let bear = find_by_name(&state, "Protected Bear");
    add_restriction(&mut state, bear, p(1), GameRestriction::CantBeSacrificed);

    let result = process_command(
        state,
        Command::ActivateAbility {
            player: p(1),
            source: altar,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: Some(bear),
            x_value: None,
        },
    );

    assert!(
        result.is_err(),
        "a can't-be-sacrificed creature must not be a legal sacrifice-filter cost target"
    );
}

/// CR 701.21a — Living Death's board-wipe step 2 ("each player sacrifices all
/// creatures they control") leaves a can't-be-sacrificed creature untouched.
#[test]
fn test_cant_be_sacrificed_board_wipe_all_creatures() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Protected Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .object(ObjectSpec::creature(p(1), "Normal Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let bear = find_by_name(&state, "Protected Bear");
    add_restriction(&mut state, bear, p(1), GameRestriction::CantBeSacrificed);

    let (state, _events) = run_effect(state, p(1), Effect::LivingDeath);

    assert!(
        on_battlefield(&state, "Protected Bear"),
        "a can't-be-sacrificed creature must survive Living Death's sacrifice-all step"
    );
    assert!(
        in_graveyard(&state, "Normal Bear", p(1)),
        "an unprotected creature must be sacrificed by the board wipe"
    );
}

/// Negative control: without the restriction, a normal permanent IS sacrificed by
/// an edict as usual.
#[test]
fn test_cant_be_sacrificed_negative_normal_permanent_is_sacrificed() {
    let state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Normal Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();

    let (state, _events) = run_effect(
        state,
        p(1),
        Effect::SacrificePermanents {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
            filter: None,
        },
    );

    assert!(
        !on_battlefield(&state, "Normal Bear"),
        "an unrestricted permanent should be sacrificed normally"
    );
    assert!(in_graveyard(&state, "Normal Bear", p(1)));
}

/// Hazard F mutation-verification: registering a `CantBeSacrificed` restriction
/// changes the public state hash.
#[test]
fn test_hash_differs_when_cant_be_sacrificed_registered() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .build()
        .unwrap();
    let bear = find_by_name(&state, "Bear");

    let hash_before = state.public_state_hash();
    add_restriction(&mut state, bear, p(1), GameRestriction::CantBeSacrificed);
    let hash_after = state.public_state_hash();

    assert_ne!(
        hash_before, hash_after,
        "registering a CantBeSacrificed restriction must change the state hash"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// GameRestriction::CantAttackOwner (CR 508.1c)
// ═══════════════════════════════════════════════════════════════════════════

/// CR 508.1c — an attacker with CantAttackOwner declaring an attack on its OWNER
/// is illegal, even though the attacker's CONTROLLER differs from its owner
/// (Alexios, Deimos of Kosmos-style control-changing creature).
#[test]
fn test_cant_attack_owner_illegal_declaration() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1)) // owner
        .add_player(p(2)) // controller / active player
        .object(
            ObjectSpec::creature(p(1), "Alexios-Style", 3, 3)
                .controlled_by(p(2))
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(2));

    let alexios = find_by_name(&state, "Alexios-Style");
    add_restriction(&mut state, alexios, p(2), GameRestriction::CantAttackOwner);

    let result = process_command(
        state,
        declare_cmd(p(2), vec![(alexios, AttackTarget::Player(p(1)))]),
    );

    assert!(
        result.is_err(),
        "creature with CantAttackOwner must not be able to attack its owner (CR 508.1c)"
    );
}

/// CR 508.1c — the same creature CAN attack a non-owner player.
#[test]
fn test_cant_attack_owner_can_attack_other_player() {
    let mut state = GameStateBuilder::four_player()
        .object(
            ObjectSpec::creature(p(1), "Alexios-Style", 3, 3)
                .controlled_by(p(2))
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(2));

    let alexios = find_by_name(&state, "Alexios-Style");
    add_restriction(&mut state, alexios, p(2), GameRestriction::CantAttackOwner);

    let result = process_command(
        state,
        declare_cmd(p(2), vec![(alexios, AttackTarget::Player(p(3)))]),
    );

    assert!(
        result.is_ok(),
        "creature with CantAttackOwner can still attack a non-owner player: {:?}",
        result.err()
    );
}

/// CR 508.1d — a requirement is obeyed only to the extent it doesn't violate a
/// restriction. A MustAttackEachCombat + CantAttackOwner creature whose ONLY
/// opponent is its owner has no legal attack target at all, so it is NOT forced
/// to attack (declaring zero attackers is legal).
#[test]
fn test_cant_attack_owner_yields_mustattack_requirement() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1)) // owner -- the only other player in this game
        .add_player(p(2)) // controller / active player
        .object(
            ObjectSpec::creature(p(1), "Alexios-Style", 3, 3)
                .controlled_by(p(2))
                .in_zone(ZoneId::Battlefield)
                .with_keyword(KeywordAbility::MustAttackEachCombat),
        )
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(2));

    let alexios = find_by_name(&state, "Alexios-Style");
    add_restriction(&mut state, alexios, p(2), GameRestriction::CantAttackOwner);

    // Declare NO attackers -- must be legal because the creature's only possible
    // target (its owner, P1) is forbidden by CantAttackOwner (CR 508.1d).
    let result = process_command(state, declare_cmd(p(2), vec![]));

    assert!(
        result.is_ok(),
        "must-attack creature with no legal target (owner-only opponent forbidden \
         by CantAttackOwner) must NOT be forced to attack (CR 508.1d): {:?}",
        result.err()
    );
}

/// Hash arm coverage: `ActiveRestriction` with `CantAttackOwner` vs
/// `CantBeSacrificed` must hash differently (disc 9 vs disc 10).
#[test]
fn test_hash_distinguishes_restriction_variants() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let src = ObjectId(1);
    let cant_attack = ActiveRestriction {
        source: src,
        controller: p(1),
        restriction: GameRestriction::CantAttackOwner,
    };
    let cant_sac = ActiveRestriction {
        source: src,
        controller: p(1),
        restriction: GameRestriction::CantBeSacrificed,
    };

    let hash_of = |r: &ActiveRestriction| -> [u8; 32] {
        let mut hasher = Hasher::new();
        r.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    assert_ne!(
        hash_of(&cant_attack),
        hash_of(&cant_sac),
        "CantAttackOwner and CantBeSacrificed must hash differently"
    );
}

/// Hash arm coverage: `Effect::WinGame` vs `Effect::LivingDeath` (both no-payload
/// variants) must hash differently (disc 90 vs disc 80).
#[test]
fn test_hash_distinguishes_wingame_effect() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let hash_of = |e: &Effect| -> [u8; 32] {
        let mut hasher = Hasher::new();
        e.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    assert_ne!(
        hash_of(&Effect::WinGame),
        hash_of(&Effect::LivingDeath),
        "Effect::WinGame and Effect::LivingDeath must hash differently"
    );
}

/// Hash arm coverage: `LossReason::OpponentWonGame` (disc 5) must hash
/// differently from `LossReason::Conceded` (disc 4).
#[test]
fn test_hash_distinguishes_loss_reason_opponent_won_game() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    let hash_of = |r: &mtg_engine::LossReason| -> [u8; 32] {
        let mut hasher = Hasher::new();
        r.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    assert_ne!(
        hash_of(&mtg_engine::LossReason::OpponentWonGame),
        hash_of(&mtg_engine::LossReason::Conceded),
        "OpponentWonGame and Conceded must hash differently"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Cleanup discard layer-correctness bug fix
// ═══════════════════════════════════════════════════════════════════════════

/// PB-AC8 bug fix regression: a permanent granted `NoMaxHandSize` by a LAYER
/// effect (`LayerModification::AddKeyword`), not printed on the card, must still
/// cause its controller to skip the cleanup discard. Pre-fix, the cleanup scan
/// read `obj.characteristics.keywords` directly and missed layer-granted keywords
/// (e.g. Wrenn and Seven's emblem proxy, `wrenn_and_seven.rs:92`).
#[test]
fn test_cleanup_layer_granted_no_max_hand_size_skips_discard() {
    use mtg_engine::{
        ContinuousEffect, EffectDuration, EffectFilter, EffectId, EffectLayer, LayerModification,
    };

    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .object(ObjectSpec::creature(p(1), "Emblem Proxy Bear", 2, 2).in_zone(ZoneId::Battlefield))
        .active_player(p(1));

    // Put 10 cards in P1's hand (over the default max hand size of 7).
    for i in 0..10 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Filler {}", i)).in_zone(ZoneId::Hand(p(1))));
    }
    let mut state = builder.build().unwrap();

    let bear = find_by_name(&state, "Emblem Proxy Bear");
    // Grant NoMaxHandSize via a Layer 6 continuous effect (mirrors the emblem's
    // AddKeyword grant), NOT a printed keyword on the card.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(1),
        source: Some(bear),
        timestamp: 10,
        layer: EffectLayer::Ability,
        duration: EffectDuration::Indefinite,
        filter: EffectFilter::SingleObject(bear),
        modification: LayerModification::AddKeyword(KeywordAbility::NoMaxHandSize),
        is_cda: false,
        condition: None,
    });

    let hand_before = state
        .zone(&ZoneId::Hand(p(1)))
        .map(|z| z.len())
        .unwrap_or(0);
    assert!(
        hand_before > 7,
        "test setup: hand must exceed max hand size"
    );

    let _events = mtg_engine::rules::turn_actions::cleanup_actions(&mut state);

    let hand_after = state
        .zone(&ZoneId::Hand(p(1)))
        .map(|z| z.len())
        .unwrap_or(0);
    assert_eq!(
        hand_after, hand_before,
        "layer-granted NoMaxHandSize must cause cleanup to skip the discard entirely"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// PB-AC8 review E1 fix — CantBeSacrificed at delayed self-sacrifice sites
// (Blitz, Mobilize) and E2 fix — Goad yields to CantAttackOwner (CR 508.1d).
// Plus T1 — CantBeSacrificed at a cast-time additional-cost site (Emerge).
// ═══════════════════════════════════════════════════════════════════════════

/// Drain the stack completely (pass all until stack is empty).
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut limit = 200;
    while !state.stack_objects().is_empty() && limit > 0 {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
        limit -= 1;
    }
    (state, all_events)
}

/// Advance to a specific step by passing priority repeatedly.
fn advance_to_step(
    mut state: GameState,
    target: Step,
    players: &[PlayerId],
) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut limit = 200;
    while state.turn().step != target && limit > 0 {
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
        limit -= 1;
    }
    (state, all_events)
}

/// CR 701.21a / CR 702.152a — a "can't be sacrificed" creature cast with Blitz is
/// NOT sacrificed by Blitz's end-step delayed trigger; it stays on the
/// battlefield instead of going to the graveyard. Regression test for review
/// finding E1 (`resolution.rs` Blitz `KeywordTrigger` arm).
#[test]
fn test_cant_be_sacrificed_blitz_delayed_sacrifice_skipped() {
    use mtg_engine::state::types::AltCostKind;

    let p1 = p(1);
    let p2 = p(2);

    let blitz_def = CardDefinition {
        card_id: CardId("blitz-bear".to_string()),
        name: "Blitz Bear".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Blitz {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Blitz),
            AbilityDefinition::AltCastAbility {
                kind: AltCostKind::Blitz,
                details: None,
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    };

    let registry = mtg_engine::CardRegistry::new(vec![blitz_def]);

    let goblin = ObjectSpec::card(p1, "Blitz Bear")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("blitz-bear".to_string()))
        .with_types(vec![CardType::Creature])
        .with_keyword(KeywordAbility::Blitz)
        .with_mana_cost(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(goblin)
        .active_player(p1)
        .at_step(Step::PostCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Red, 1);
    state.turn_mut().priority_holder = Some(p1);

    let card_id = find_by_name(&state, "Blitz Bear");

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: card_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Blitz),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("CastSpell with blitz failed: {:?}", e));

    // Resolve the spell -- creature enters the battlefield.
    let (mut state, _) = pass_all(state, &[p1, p2]);
    assert!(
        on_battlefield(&state, "Blitz Bear"),
        "creature should be on battlefield after blitz cast"
    );

    // Grant CantBeSacrificed AFTER the creature is on the battlefield, before
    // the end-step delayed trigger fires.
    let bear = find_by_name(&state, "Blitz Bear");
    add_restriction(&mut state, bear, p1, GameRestriction::CantBeSacrificed);

    // Advance to End step -- Blitz's delayed trigger is queued.
    let (state, _) = advance_to_step(state, Step::End, &[p1, p2]);
    assert_eq!(state.turn().step, Step::End, "should reach End step");

    // Resolve the delayed trigger. Because the creature can't be sacrificed, the
    // trigger must do nothing -- no CreatureDied event, no draw trigger, no
    // graveyard zone change.
    let (state, _end_events) = pass_all(state, &[p1, p2]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Blitz Bear"),
        "PB-AC8 review E1 / CR 701.21a: a can't-be-sacrificed creature must NOT be \
         sacrificed by Blitz's end-step delayed trigger -- it stays on the battlefield"
    );
    assert!(
        !in_graveyard(&state, "Blitz Bear", p1),
        "protected creature must not have been moved to the graveyard"
    );
}

/// CR 701.21a / CR 603.7 — a "can't be sacrificed" Mobilize-style token
/// (`sacrifice_at_end_step` flag) is NOT sacrificed at the next end step; it
/// stays on the battlefield. Regression test for review finding E1
/// (`turn_actions.rs` / `resolution.rs` shared `DelayedTriggerAction::SacrificeObject`
/// dispatch).
#[test]
fn test_cant_be_sacrificed_mobilize_delayed_sacrifice_skipped() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let token_spec = mtg_engine::cards::card_definition::TokenSpec {
        name: "Mobilized Warrior".to_string(),
        card_types: [CardType::Creature].into_iter().collect(),
        subtypes: [mtg_engine::state::types::SubType("Warrior".to_string())]
            .into_iter()
            .collect(),
        colors: [mtg_engine::state::types::Color::Red].into_iter().collect(),
        power: 1,
        toughness: 1,
        count: EffectAmount::Fixed(1),
        sacrifice_at_end_step: true,
        ..Default::default()
    };
    let token_obj = mtg_engine::effects::make_token(&token_spec, p1);
    test_util::add_object(&mut state, token_obj, ZoneId::Battlefield).expect("add token");

    assert!(
        on_battlefield(&state, "Mobilized Warrior"),
        "token on battlefield before end step"
    );

    let token = find_by_name(&state, "Mobilized Warrior");
    add_restriction(&mut state, token, p1, GameRestriction::CantBeSacrificed);

    let (state, _) = advance_to_step(state, Step::End, &[p1, p2]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Mobilized Warrior"),
        "PB-AC8 review E1 / CR 701.21a: a can't-be-sacrificed Mobilize token must \
         NOT be sacrificed at the next end step -- it stays on the battlefield"
    );
}

/// CR 508.1d — a requirement is obeyed only to the extent it doesn't violate a
/// restriction. A GOADED creature (CR 701.15b requirement) with CantAttackOwner
/// (CR 508.1c restriction) whose ONLY opponent is its owner has no legal attack
/// target at all, so goad's "must attack if able" does NOT force it. Regression
/// test for review finding E2 (`combat.rs` goad "must attack" block, which had
/// the owner-exclusion applied only to `MustAttackEachCombat`, not to goad).
#[test]
fn test_cant_attack_owner_goad_yields_requirement() {
    let mut state = GameStateBuilder::new()
        .add_player(p(1)) // owner -- the only other player in this game
        .add_player(p(2)) // controller / active player
        .object(
            ObjectSpec::creature(p(1), "Goaded Alexios-Style", 3, 3)
                .controlled_by(p(2))
                .in_zone(ZoneId::Battlefield),
        )
        .active_player(p(2))
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();
    state.turn_mut().priority_holder = Some(p(2));

    let creature = find_by_name(&state, "Goaded Alexios-Style");
    add_restriction(&mut state, creature, p(2), GameRestriction::CantAttackOwner);
    // Goad the creature (the goading player's identity doesn't matter for the
    // "must attack if able" computation -- only that `goaded_by` is non-empty).
    state
        .objects_mut()
        .get_mut(&creature)
        .unwrap()
        .goaded_by
        .push_back(p(1));

    // Declare NO attackers -- must be legal because the creature's only possible
    // target (its owner, P1) is forbidden by CantAttackOwner (CR 508.1d).
    let result = process_command(state, declare_cmd(p(2), vec![]));

    assert!(
        result.is_ok(),
        "goaded creature with no legal target (owner-only opponent forbidden by \
         CantAttackOwner) must NOT be forced to attack (CR 508.1d): {:?}",
        result.err()
    );
}

/// T1 — CR 701.21a / CR 702.119a: a "can't be sacrificed" creature cannot be
/// named as the sacrifice for a cast-time additional cost (Emerge). Closes the
/// review's cast-cost test gap (the guards existed at `casting.rs` for Emerge/
/// Bargain/Casualty/SpellAdditionalCost/Devour, but none were exercised by a test).
#[test]
fn test_cant_be_sacrificed_cast_cost_emerge_cannot_pay() {
    use mtg_engine::state::types::AltCostKind;

    let p1 = p(1);
    let p2 = p(2);

    let emerge_def = CardDefinition {
        card_id: CardId("emerge-test-creature".to_string()),
        name: "Emerge Test Creature".to_string(),
        mana_cost: Some(ManaCost {
            generic: 8,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        power: Some(5),
        toughness: Some(6),
        oracle_text: "Emerge {5}{U}{U}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Emerge),
            AbilityDefinition::Emerge {
                cost: ManaCost {
                    generic: 5,
                    blue: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    };

    let registry = mtg_engine::CardRegistry::new(vec![emerge_def]);

    let spell = ObjectSpec::card(p1, "Emerge Test Creature")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("emerge-test-creature".to_string()))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 8,
            ..Default::default()
        })
        .with_keyword(KeywordAbility::Emerge);

    let sac_creature =
        ObjectSpec::creature(p1, "Protected Sac Fodder", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(sac_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 2);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Blue, 2);
    state.turn_mut().priority_holder = Some(p1);

    let spell_id = find_by_name(&state, "Emerge Test Creature");
    let sac_id = find_by_name(&state, "Protected Sac Fodder");
    add_restriction(&mut state, sac_id, p1, GameRestriction::CantBeSacrificed);

    let result = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: spell_id,
            targets: vec![],
            convoke_creatures: vec![],
            improvise_artifacts: vec![],
            delve_cards: vec![],
            kicker_times: 0,
            alt_cost: Some(AltCostKind::Emerge),
            prototype: false,
            modes_chosen: vec![],
            x_value: 0,
            face_down_kind: None,
            additional_costs: vec![mtg_engine::AdditionalCost::Sacrifice {
                ids: vec![sac_id],
                lki_powers: vec![],
            }],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    );

    assert!(
        result.is_err(),
        "a can't-be-sacrificed creature must not be a legal Emerge sacrifice cost \
         target (CR 701.21a / CR 702.119a)"
    );
}

/// CR 701.21a / CR 702.147a — a "can't be sacrificed" creature that attacked with
/// Decayed is NOT sacrificed at end of combat; it stays on the battlefield.
/// Regression test for review finding E1 (`turn_actions.rs` decayed end-of-combat
/// TBA sacrifice, the fourth flagged site alongside Blitz/Encore/Mobilize).
#[test]
fn test_cant_be_sacrificed_decayed_end_of_combat_sacrifice_skipped() {
    let p1 = p(1);
    let p2 = p(2);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(
            ObjectSpec::creature(p1, "Decayed Protected Attacker", 2, 2)
                .with_keyword(KeywordAbility::Decayed),
        )
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_by_name(&state, "Decayed Protected Attacker");

    // Declare the decayed creature as attacker (this sets decayed_sacrifice_at_eoc).
    let (mut state, _) = process_command(
        state,
        declare_cmd(p1, vec![(attacker_id, AttackTarget::Player(p2))]),
    )
    .expect("DeclareAttackers should succeed");

    // Grant CantBeSacrificed on the (possibly re-identified) attacker object.
    let attacker_id = find_by_name(&state, "Decayed Protected Attacker");
    add_restriction(
        &mut state,
        attacker_id,
        p1,
        GameRestriction::CantBeSacrificed,
    );

    // Advance to DeclareBlockers, declare no blockers, then advance through
    // CombatDamage into EndOfCombat -- `end_combat()` runs on step entry and is
    // where the decayed sacrifice normally happens (turn_actions.rs dispatches
    // `Step::EndOfCombat => end_combat(state)`).
    let (state, _) = advance_to_step(state, Step::DeclareBlockers, &[p1, p2]);
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("DeclareBlockers (empty) should succeed");
    let (state, _) = advance_to_step(state, Step::EndOfCombat, &[p1, p2]);

    assert!(
        on_battlefield(&state, "Decayed Protected Attacker"),
        "PB-AC8 review E1 / CR 701.21a: a can't-be-sacrificed creature must NOT be \
         sacrificed by decayed's end-of-combat trigger -- it stays on the battlefield"
    );
    assert!(
        !in_graveyard(&state, "Decayed Protected Attacker", p1),
        "protected decayed attacker must not have been moved to the graveyard"
    );
}
