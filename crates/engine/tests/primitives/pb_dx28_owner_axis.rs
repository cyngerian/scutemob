//! PB-DX28 §2/§3 — the owner axis (`OOS-DX4-1`) and `sword_of_war_and_peace` /
//! `EffectTarget::DamagedPlayer`.
//!
//! `memory/primitives/pb-plan-DX28.md` §2/§3 is authoritative for this file. Covers:
//!
//! - **A**: `TargetFilter.owner` (CR 108.3) enforced at
//!   `casting::validate_object_satisfies_requirement` (the declarative cast/activation
//!   path, `TargetPermanentWithFilter`) — the two control-change directions, plus
//!   `TargetOwner::Opponent` and the default `TargetOwner::Any`.
//! - **B**: `TriggerCondition::WheneverCreatureDies.owner` lowered into
//!   `DeathTriggerFilter.owner_you`/`owner_opponent` and enforced at the BATTLEFIELD
//!   dispatch site (`rules::abilities`'s `AnyCreatureDies` loop) — same two directions,
//!   plus `Some(TargetOwner::Opponent)` and `Some(TargetOwner::Any)` (behaviorally
//!   identical to `None`, proven rather than assumed).
//! - **C**: the same field on the GRAVEYARD-zone dispatch site
//!   (`collect_graveyard_carddef_triggers`), exercised through the REAL
//!   `nether_traitor()` card def (CR 108.3 / CR 404.3 / CR 113.6m).
//! - **D**: `EffectTarget::DamagedPlayer` (CR 510.3a / CR 115.10), resolved from
//!   `EffectContext::damaged_player` — a 4-player board where the damaged player is
//!   NOT the first opponent in `turn_order`, both as a direct synthetic probe and as a
//!   card-integration test through the real `sword_of_war_and_peace()` def.
//!
//! CR rules covered: 108.3 (ownership), 109.4 (control), 400.7 (zone-change identity —
//! ownership is invariant across it, controller is not), 404.3 (graveyard ownership),
//! 113.6m (Nether Traitor functions only from the graveyard), 510.3a (damaged-player
//! context), 601.2c (declarative target validation), 603.10a (death-trigger look-back).

use mtg_engine::cards::card_definition::Cost;
use mtg_engine::state::stubs::PendingTriggerKind;
use mtg_engine::{
    all_cards, card_name_to_id, check_and_apply_sbas, enrich_spec_from_def, process_command,
    AbilityDefinition, AttackTarget, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    CardType, Command, DeathTriggerFilter, Effect, EffectAmount, GameEvent, GameState,
    GameStateBuilder, GameStateError, ManaColor, ObjectId, ObjectSpec, PlayerId, PlayerTarget,
    Step, Target, TargetFilter, TargetOwner, TargetRequirement, TriggerCondition, TriggerEvent,
    TriggeredAbilityDef, TypeLine, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_graveyard(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == name && matches!(o.zone, ZoneId::Graveyard(_)))
        .map(|(&id, _)| id)
        .unwrap_or_else(|| panic!("object '{name}' not found in any graveyard"))
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .into_iter()
        .map(|d| (d.name.clone(), d))
        .collect()
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

// ── Section A: TargetFilter.owner at the declarative cast/activation path ─────

/// A creature with `{T}: Destroy target permanent [owner scope].` — the exact shape
/// `staff_of_compleation.rs` uses (minus the mana/life cost, irrelevant here).
fn owner_scoped_destroyer(name: &str, owner: TargetOwner) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-owner-scoped-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::DestroyPermanent {
                target: CardEffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: true,
            },
            timing_restriction: None,
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                owner,
                ..Default::default()
            })],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        ..Default::default()
    }
}

/// Two candidate permanents plus one `owner_scoped_destroyer(owner)`, all controlled
/// by p1 except where the candidate's own owner/controller split says otherwise.
/// "Owned P1 Ctrl P2": owner p1, controller p2 (a Control-Magic-style split).
/// "Owned P2 Ctrl P1": owner p2, controller p1.
fn build_owner_axis_board(destroyer_owner_scope: TargetOwner) -> (GameState, PlayerId, PlayerId) {
    let p1 = p(1);
    let p2 = p(2);

    let destroyer_def = owner_scoped_destroyer("Reclaimer", destroyer_owner_scope);
    let destroyer = enrich_spec_from_def(
        ObjectSpec::card(p1, "Reclaimer")
            .with_card_id(destroyer_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &HashMap::from([(destroyer_def.name.clone(), destroyer_def.clone())]),
    );

    let mut owned_p1_ctrl_p2 = ObjectSpec::creature(p1, "Owned P1 Ctrl P2", 2, 2);
    owned_p1_ctrl_p2.controller = Some(p2);
    let mut owned_p2_ctrl_p1 = ObjectSpec::creature(p2, "Owned P2 Ctrl P1", 2, 2);
    owned_p2_ctrl_p1.controller = Some(p1);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![destroyer_def]))
        .object(destroyer)
        .object(owned_p1_ctrl_p2)
        .object(owned_p2_ctrl_p1)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("builder must succeed");

    (state, p1, p2)
}

fn activate_reclaimer(
    state: GameState,
    source: ObjectId,
    target: ObjectId,
    player: PlayerId,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source,
            ability_index: 0,
            targets: vec![Target::Object(target)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
}

/// CR 108.3: a permanent P1 OWNS but P2 CONTROLS is a LEGAL target for
/// `owner: TargetOwner::You` — the direction the pre-batch `TargetController::You`
/// approximation (`staff_of_compleation`'s pre-fix shape) got wrong.
#[test]
fn t1_owner_you_accepts_owned_but_opponent_controlled() {
    let (state, p1, _p2) = build_owner_axis_board(TargetOwner::You);
    let reclaimer_id = find_obj(&state, "Reclaimer");
    let target_id = find_obj(&state, "Owned P1 Ctrl P2");

    let result = activate_reclaimer(state, reclaimer_id, target_id, p1);
    assert!(
        result.is_ok(),
        "CR 108.3: TargetOwner::You must accept a permanent P1 OWNS even though P2 \
         controls it: {:?}",
        result.err()
    );
}

/// CR 108.3: a permanent P1 CONTROLS but does NOT OWN is an ILLEGAL target for
/// `owner: TargetOwner::You` — the direction the pre-batch `TargetController::You`
/// approximation got wrong the other way.
#[test]
fn t2_owner_you_rejects_controlled_but_not_owned() {
    let (state, p1, _p2) = build_owner_axis_board(TargetOwner::You);
    let reclaimer_id = find_obj(&state, "Reclaimer");
    let target_id = find_obj(&state, "Owned P2 Ctrl P1");

    let result = activate_reclaimer(state, reclaimer_id, target_id, p1);
    assert!(
        result.is_err(),
        "CR 108.3: TargetOwner::You must reject a permanent P1 CONTROLS but does not \
         OWN"
    );
    assert!(matches!(
        result.unwrap_err(),
        GameStateError::InvalidTarget(_)
    ));
}

/// `TargetOwner::Opponent` is not half-dead: mirrors T1/T2 with the scope flipped.
#[test]
fn t3_owner_opponent_accepts_opponent_owned_rejects_self_owned() {
    let (state, p1, _p2) = build_owner_axis_board(TargetOwner::Opponent);
    let reclaimer_id = find_obj(&state, "Reclaimer");
    let opponent_owned = find_obj(&state, "Owned P2 Ctrl P1");
    let self_owned = find_obj(&state, "Owned P1 Ctrl P2");

    let ok = activate_reclaimer(state.clone(), reclaimer_id, opponent_owned, p1);
    assert!(
        ok.is_ok(),
        "TargetOwner::Opponent must accept a permanent the caster's opponent owns \
         (regardless of who controls it): {:?}",
        ok.err()
    );

    let err = activate_reclaimer(state, reclaimer_id, self_owned, p1);
    assert!(
        err.is_err(),
        "TargetOwner::Opponent must reject a permanent the CASTER owns"
    );
}

/// The default `TargetOwner::Any` narrows nothing: both the owned-by-caster and
/// owned-by-opponent permanents are legal targets, proving the enum's default arm is
/// not silently equivalent to `You` or `Opponent`.
#[test]
fn t4_owner_any_default_narrows_nothing() {
    let (state, p1, _p2) = build_owner_axis_board(TargetOwner::Any);
    let reclaimer_id = find_obj(&state, "Reclaimer");
    let owned_by_p1 = find_obj(&state, "Owned P1 Ctrl P2");
    let owned_by_p2 = find_obj(&state, "Owned P2 Ctrl P1");

    let a = activate_reclaimer(state.clone(), reclaimer_id, owned_by_p1, p1);
    assert!(
        a.is_ok(),
        "TargetOwner::Any (default) must accept a caster-owned permanent: {:?}",
        a.err()
    );
    let b = activate_reclaimer(state, reclaimer_id, owned_by_p2, p1);
    assert!(
        b.is_ok(),
        "TargetOwner::Any (default) must accept an opponent-owned permanent: {:?}",
        b.err()
    );
}

// ── Section B: DeathTriggerFilter.owner_you/owner_opponent, battlefield dispatch ──

/// A battlefield watcher with a raw `TriggeredAbilityDef` carrying the given
/// `owner_you`/`owner_opponent` — exercises `rules::abilities`'s `AnyCreatureDies`
/// battlefield loop directly, independent of card-def lowering (Section C covers
/// that separately, through the real `nether_traitor()` def).
fn owner_scoped_watcher(owner_you: bool, owner_opponent: bool) -> TriggeredAbilityDef {
    TriggeredAbilityDef {
        counter_filter: None,
        counter_on_self: false,
        once_per_turn: false,
        trigger_on: TriggerEvent::AnyCreatureDies,
        intervening_if: None,
        description: "Whenever a creature [owner scope] dies, draw a card.".to_string(),
        effect: Some(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
        etb_filter: None,
        death_filter: Some(DeathTriggerFilter {
            controller_you: false,
            controller_opponent: false,
            exclude_self: false,
            nontoken_only: false,
            owner_you,
            owner_opponent,
        }),
        combat_damage_filter: None,
        triggering_creature_filter: None,
        targets: vec![],
    }
}

/// Kills `fodder_owner`-owned / `fodder_controller`-controlled 0-toughness fodder via
/// `check_and_apply_sbas` directly (mirrors PB-DX24's fixture) and returns the number
/// of `AnyCreatureDies`-sourced `PendingTrigger`s at `watcher_id`.
fn kill_fodder_and_count_watcher_triggers(
    watcher_owner_you: bool,
    watcher_owner_opponent: bool,
    fodder_owner: PlayerId,
    fodder_controller: PlayerId,
) -> usize {
    let p1 = p(1);
    let p2 = p(2);

    let watcher = ObjectSpec::creature(p1, "Watcher", 2, 2)
        .with_card_id(CardId("watcher".to_string()))
        .with_triggered_ability(owner_scoped_watcher(
            watcher_owner_you,
            watcher_owner_opponent,
        ));
    let mut fodder = ObjectSpec::creature(fodder_owner, "Fodder", 1, 0);
    fodder.controller = Some(fodder_controller);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(watcher)
        .object(fodder)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let watcher_id = find_obj(&state, "Watcher");
    check_and_apply_sbas(&mut state);

    state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == watcher_id)
        .count()
}

/// CR 108.3 / 603.10a: `owner_you` fires when the dying creature is OWNED by the
/// watcher's controller even though an opponent CONTROLLED it.
#[test]
fn t5_owner_you_fires_on_owned_but_opponent_controlled_death() {
    let p1 = p(1);
    let p2 = p(2);
    let count = kill_fodder_and_count_watcher_triggers(true, false, p1, p2);
    assert_eq!(
        count, 1,
        "owner_you must fire when P1 (watcher's controller) OWNS the dying creature, \
         even though P2 controlled it"
    );
}

/// The mirror negative: `owner_you` must NOT fire when the watcher's controller
/// CONTROLS but does not OWN the dying creature.
#[test]
fn t6_owner_you_does_not_fire_on_controlled_but_not_owned_death() {
    let p1 = p(1);
    let p2 = p(2);
    let count = kill_fodder_and_count_watcher_triggers(true, false, p2, p1);
    assert_eq!(
        count, 0,
        "owner_you must NOT fire when P1 (watcher's controller) merely CONTROLS the \
         dying creature without owning it"
    );
}

/// `owner_opponent` is not half-dead: mirrors T5/T6 with the scope flipped.
#[test]
fn t7_owner_opponent_fires_on_opponent_owned_death_only() {
    let p1 = p(1);
    let p2 = p(2);
    let fires_on_p2_owned = kill_fodder_and_count_watcher_triggers(false, true, p2, p1);
    assert_eq!(
        fires_on_p2_owned, 1,
        "owner_opponent must fire when an opponent (P2) OWNS the dying creature"
    );
    let silent_on_p1_owned = kill_fodder_and_count_watcher_triggers(false, true, p1, p2);
    assert_eq!(
        silent_on_p1_owned, 0,
        "owner_opponent must NOT fire when the watcher's own controller (P1) owns the \
         dying creature"
    );
}

/// `TriggerCondition::WheneverCreatureDies.owner: Some(TargetOwner::Any)` behaves
/// identically to `None` (no ownership restriction) — the enum's `Any` arm is not
/// silently equivalent to `You`, and is reachable through the trigger path too, not
/// only through `TargetFilter.owner`.
#[test]
fn t8_trigger_owner_any_is_unrestricted_like_none() {
    // Goes through the REAL card-def lowering (`build_face_ability_vectors` in
    // `testing/replay_harness.rs`), unlike `kill_fodder_and_count_watcher_triggers`
    // above (which builds `DeathTriggerFilter` by hand and so cannot discriminate a
    // mis-lowered `Some(TargetOwner::Any)` -- a plausible bug would map it onto
    // `owner_you: true`, "Any" mistaken for "You"). `def.abilities[0]`'s
    // `TriggerCondition::WheneverCreatureDies { owner: Some(TargetOwner::Any), .. }`
    // is the exact enum value under test.
    let p1 = p(1);
    let p2 = p(2);

    let watcher_def = CardDefinition {
        name: "Any-Owner Watcher".to_string(),
        card_id: CardId("test-any-owner-watcher".to_string()),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WheneverCreatureDies {
                controller: None,
                owner: Some(TargetOwner::Any),
                exclude_self: false,
                nontoken_only: false,
                filter: None,
            },
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    };

    let count_watcher_triggers = |fodder_owner: PlayerId, fodder_controller: PlayerId| -> usize {
        let defs = HashMap::from([(watcher_def.name.clone(), watcher_def.clone())]);
        let watcher = enrich_spec_from_def(
            ObjectSpec::card(p1, "Any-Owner Watcher")
                .with_card_id(watcher_def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs,
        );
        let mut fodder = ObjectSpec::creature(fodder_owner, "Fodder", 1, 0);
        fodder.controller = Some(fodder_controller);

        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(CardRegistry::new(vec![watcher_def.clone()]))
            .object(watcher)
            .object(fodder)
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        let watcher_id = find_obj(&state, "Any-Owner Watcher");
        check_and_apply_sbas(&mut state);
        state
            .pending_triggers()
            .iter()
            .filter(|t| t.source == watcher_id)
            .count()
    };

    let fires_on_p1_owned = count_watcher_triggers(p1, p2);
    let fires_on_p2_owned = count_watcher_triggers(p2, p1);
    assert_eq!(
        fires_on_p1_owned, 1,
        "Some(TargetOwner::Any) must fire regardless of who owns the dying creature (P1 case)"
    );
    assert_eq!(
        fires_on_p2_owned, 1,
        "Some(TargetOwner::Any) must fire regardless of who owns the dying creature (P2 case)"
    );
}

// ── Section C: the graveyard-zone dispatch site, via the REAL nether_traitor() def ──

fn nether_traitor_death_ability_index(defs: &HashMap<String, CardDefinition>) -> usize {
    let def = defs.get("Nether Traitor").unwrap();
    def.abilities
        .iter()
        .position(|a| {
            matches!(
                a,
                AbilityDefinition::Triggered {
                    trigger_condition: TriggerCondition::WheneverCreatureDies { .. },
                    ..
                }
            )
        })
        .expect("Nether Traitor must have a WheneverCreatureDies ability")
}

/// Nether Traitor in p1's OWN graveyard, plus a 0-toughness Fodder creature owned by
/// `fodder_owner` and controlled by `fodder_controller`.
fn build_nether_owner_axis_fixture(
    defs: &HashMap<String, CardDefinition>,
    fodder_owner: PlayerId,
    fodder_controller: PlayerId,
) -> GameState {
    let p1 = p(1);
    let nether_card_id = defs.get("Nether Traitor").unwrap().card_id.clone();
    let nether_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Nether Traitor")
            .in_zone(ZoneId::Graveyard(p1))
            .with_card_id(nether_card_id),
        defs,
    );
    let mut fodder = ObjectSpec::creature(fodder_owner, "Fodder", 1, 0);
    fodder.controller = Some(fodder_controller);

    let defs_vec: Vec<CardDefinition> = defs.values().cloned().collect();
    GameStateBuilder::four_player()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .with_registry(CardRegistry::new(defs_vec))
        .object(nether_spec)
        .object(fodder)
        .build()
        .unwrap()
}

/// CR 108.3 / CR 404.3 / CR 113.6m — Nether Traitor's printed clause is "another
/// creature is put into YOUR graveyard" (ownership). A creature P1 OWNS but P2
/// CONTROLS dying (into P1's graveyard, per CR 400.7/404.3 — a dying creature always
/// enters its OWNER's graveyard) must fire the trigger. Pre-fix (controller-scoped
/// approximation) this did NOT fire, because the pre-death CONTROLLER (P2) was
/// compared against Nether Traitor's owner (P1) and they differed.
#[test]
fn t9_nether_traitor_fires_when_owned_but_opponent_controlled_creature_dies() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();
    let expected_ability_index = nether_traitor_death_ability_index(&defs);

    let mut state = build_nether_owner_axis_fixture(&defs, p1, p2);
    let nether_gy_id = find_obj(&state, "Nether Traitor");

    check_and_apply_sbas(&mut state);

    let fodder_new_id = find_in_graveyard(&state, "Fodder");
    let nether_triggers: Vec<_> = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_gy_id)
        .collect();

    assert_eq!(
        nether_triggers.len(),
        1,
        "CR 108.3/404.3: Nether Traitor must trigger when a creature P1 OWNS (but P2 \
         controlled) dies into P1's own graveyard. Got {} trigger(s): {:?}",
        nether_triggers.len(),
        nether_triggers
    );
    let t = nether_triggers[0];
    assert_eq!(t.kind, PendingTriggerKind::CardDefETB);
    assert_eq!(t.triggering_event, Some(TriggerEvent::AnyCreatureDies));
    assert_eq!(t.entering_object_id, Some(fodder_new_id));
    assert_eq!(t.ability_index, expected_ability_index);
}

/// The mirror negative: a creature P1 CONTROLS but does NOT OWN (owner P2) dies into
/// P2's graveyard, not P1's — Nether Traitor (sitting in P1's graveyard) must NOT
/// fire. Pre-fix (controller-scoped approximation) this WOULD have fired, because the
/// pre-death CONTROLLER (P1) was compared against Nether Traitor's owner (P1) and
/// they matched, even though ownership (and therefore which graveyard the card
/// actually entered) did not.
#[test]
fn t10_nether_traitor_does_not_fire_when_controlled_but_not_owned_creature_dies() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let mut state = build_nether_owner_axis_fixture(&defs, p2, p1);
    let nether_gy_id = find_obj(&state, "Nether Traitor");

    check_and_apply_sbas(&mut state);

    // Non-vacuity: the creature really did die, into P2's graveyard (not P1's).
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Fodder" && o.zone == ZoneId::Graveyard(p2)),
        "non-vacuity: Fodder must have died into P2's (its owner's) graveyard"
    );

    let nether_triggers: Vec<_> = state
        .pending_triggers()
        .iter()
        .filter(|t| t.source == nether_gy_id)
        .collect();
    assert!(
        nether_triggers.is_empty(),
        "CR 108.3/404.3: Nether Traitor must NOT trigger when a creature P1 merely \
         CONTROLLED (owner P2) dies into P2's graveyard, not P1's. Got {} trigger(s): \
         {:?}",
        nether_triggers.len(),
        nether_triggers
    );
}

// ── Section D: EffectTarget::DamagedPlayer ─────────────────────────────────────

/// A self-referential "whenever this deals combat damage to a player, deal N damage
/// to that player" creature — isolates `EffectTarget::DamagedPlayer` resolution from
/// Equipment/attach mechanics entirely.
fn damaged_player_avenger(name: &str, bonus_damage: i32) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-damaged-player-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Creature],
            ..Default::default()
        },
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
            effect: Effect::DealDamage {
                source: None,
                target: CardEffectTarget::DamagedPlayer,
                amount: EffectAmount::Fixed(bonus_damage),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}

/// CR 510.3a / CR 115.10: in a 4-player game (turn order p1, p2, p3, p4), p1's
/// creature attacks p3 DIRECTLY (skipping p2, the "first opponent" in seat order).
/// `EffectTarget::DamagedPlayer` must resolve to p3 — the player who was ACTUALLY
/// damaged — not to p2. This is the exact defect class `sword_of_war_and_peace` had:
/// a `TargetPlayer` requirement resolved through the CR 601.2c auto-target picker
/// would choose *a* player (often the first opponent), not necessarily the damaged
/// one.
#[test]
fn t11_damaged_player_resolves_to_the_actually_damaged_seat_not_seat_order() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let avenger_def = damaged_player_avenger("Avenger", 3);
    let avenger = enrich_spec_from_def(
        ObjectSpec::card(p1, "Avenger")
            .with_card_id(avenger_def.card_id.clone())
            .in_zone(ZoneId::Battlefield),
        &HashMap::from([(avenger_def.name.clone(), avenger_def.clone())]),
    );

    let state = GameStateBuilder::four_player()
        .with_registry(CardRegistry::new(vec![avenger_def]))
        .object(avenger)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let avenger_id = find_obj(&state, "Avenger");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(avenger_id, AttackTarget::Player(p3))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("DeclareAttackers");

    // Advance through DeclareBlockers (unblocked, no blockers exist) into combat
    // damage, then resolve the triggered ability.
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let life = |s: &GameState, pl: PlayerId| s.players().get(&pl).unwrap().life_total;

    assert_eq!(
        life(&state, p2),
        40,
        "p2 (turn-order-first opponent, but NOT the damaged player) must be untouched"
    );
    assert_eq!(
        life(&state, p4),
        40,
        "p4 (also not the damaged player) must be untouched"
    );
    assert_eq!(
        life(&state, p3),
        40 - 2 - 3,
        "p3 (the ACTUALLY damaged player) must take both the base combat damage (2) \
         and the triggered ability's bonus damage (3), routed through \
         EffectTarget::DamagedPlayer"
    );
}

/// Card-integration test: the real `sword_of_war_and_peace()` def, equipped and
/// attacking a non-adjacent-in-seat-order player in a 4-player game. Proves the
/// repaired def (targets: vec![], both `EffectTarget::DamagedPlayer` and
/// `PlayerTarget::DamagedPlayer` reads) works end to end through the real Equip
/// activation and the real `WhenEquippedCreatureDealsCombatDamageToPlayer` dispatch,
/// not just the isolated primitive.
#[test]
fn t12_sword_of_war_and_peace_damages_the_actually_damaged_player() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = load_defs();

    let sword_card_id = card_name_to_id("Sword of War and Peace");
    let sword = enrich_spec_from_def(
        ObjectSpec::card(p1, "Sword of War and Peace")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(sword_card_id.clone()),
        &defs,
    );
    let bearer = ObjectSpec::creature(p1, "Bearer", 2, 2);

    // p1 gets 2 hand cards (for GainLife), p3 (the target) gets 1 (for DealDamage's
    // bonus amount) — distinguishable, non-zero, non-equal so a swapped-argument bug
    // would be visible.
    let p1_card1 = ObjectSpec::card(p1, "P1 Hand Card 1")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));
    let p1_card2 = ObjectSpec::card(p1, "P1 Hand Card 2")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p1));
    let p3_card1 = ObjectSpec::card(p3, "P3 Hand Card 1")
        .with_types(vec![CardType::Instant])
        .in_zone(ZoneId::Hand(p3));

    let mut state = GameStateBuilder::four_player()
        .with_registry(CardRegistry::new(
            defs.values().cloned().collect::<Vec<_>>(),
        ))
        .object(sword)
        .object(bearer)
        .object(p1_card1)
        .object(p1_card2)
        .object(p3_card1)
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

    let sword_id = find_obj(&state, "Sword of War and Peace");
    let bearer_id = find_obj(&state, "Bearer");

    // `GameStateBuilder::object()` places the Sword directly on the battlefield
    // without running ETB machinery, so its three `AbilityDefinition::Static`
    // entries (+2/+2, protection from red, protection from white) are never
    // registered into `state.continuous_effects` -- mirrors
    // `cards1_equip_target_repair.rs`'s identical Skullclamp gotcha.
    let sword_card_id_present = state
        .objects()
        .get(&sword_id)
        .and_then(|o| o.card_id.clone());
    let registry = state.card_registry().clone();
    mtg_engine::rules::replacement::register_static_continuous_effects(
        &mut state,
        sword_id,
        sword_card_id_present.as_ref(),
        &registry,
        false,
    );

    // Derive the Equip ability's RUNTIME activated-ability index (never hard-coded):
    // the def's only `AbilityDefinition::Activated` variant.
    let sword_def = defs.get("Sword of War and Peace").unwrap();
    let equip_index = sword_def
        .abilities
        .iter()
        .filter(|a| matches!(a, AbilityDefinition::Activated { .. }))
        .position(|a| {
            matches!(
                a,
                AbilityDefinition::Activated {
                    effect: Effect::AttachEquipment { .. },
                    ..
                }
            )
        })
        .expect("Sword of War and Peace must have an AttachEquipment activated ability");

    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: sword_id,
            ability_index: equip_index,
            targets: vec![Target::Object(bearer_id)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("Equip {2} must succeed");

    // Advance to DeclareAttackers (bounded loop: each round of all-pass advances one step).
    let mut state = state;
    let mut guard = 0;
    while state.turn().step != Step::DeclareAttackers {
        state = pass_all(state, &[p1, p2, p3, p4]).0;
        guard += 1;
        assert!(
            guard < 10,
            "did not reach DeclareAttackers after equipping within 10 pass rounds \
             (stuck at {:?})",
            state.turn().step
        );
    }

    let bearer_id = find_obj(&state, "Bearer");
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(bearer_id, AttackTarget::Player(p3))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("DeclareAttackers");

    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);
    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    let life = |s: &GameState, pl: PlayerId| s.players().get(&pl).unwrap().life_total;

    assert_eq!(
        life(&state, p2),
        40,
        "p2 (turn-order-first opponent, but not the damaged player) must be untouched"
    );
    assert_eq!(
        life(&state, p4),
        40,
        "p4 (also not the damaged player) must be untouched"
    );
    // Bearer is 2/2 base + Sword's +2/+2 = 4/4 combat damage, plus the triggered
    // ability's damage equal to p3's hand size (1 card).
    assert_eq!(
        life(&state, p3),
        40 - 4 - 1,
        "p3 (the actually-damaged player) must take combat damage (4, boosted by the \
         Sword) plus the triggered ability's damage equal to THEIR OWN hand size (1) \
         -- not p2's"
    );
    assert_eq!(
        life(&state, p1),
        // GainLife equal to the CONTROLLER's (p1's) own hand size at resolution. p1's
        // hand started at 2 and is unchanged by this sequence (no draws/discards).
        40 + 2,
        "p1 (the controller) must gain life equal to their OWN hand size (2), via \
         PlayerTarget::Controller -- unrelated to DamagedPlayer, checked here for \
         completeness of the card-integration test"
    );
}
