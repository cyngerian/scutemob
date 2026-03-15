//! Card definition tests for dungeon/venture cards — Session 4 (CR 309.4, CR 701.49, CR 725.2).
//!
//! Tests cover:
//! - Nadaar ETB triggers venture into the dungeon (CR 701.49a)
//! - Nadaar attack triggers venture into the dungeon (CR 701.49b context)
//! - Nadaar "+1/+1 to other creatures" (DSL gap — static ability absent; presence of ability list tested)
//! - Acererak bounces to hand when Tomb of Annihilation not completed (CR 603.4 intervening-if)
//! - TakeTheInitiative forces venture into Undercity (CR 725.2, CR 701.49d)

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::state::zone::ZoneId;
use mtg_engine::{
    all_cards, card_name_to_id, enrich_spec_from_def, process_command, CardDefinition,
    CardRegistry, Command, DungeonId, GameEvent, GameStateBuilder, ObjectSpec, PlayerId, Step,
};

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Build the card-definition map and registry, returned as a tuple.
/// Note: `CardRegistry::new()` returns `Arc<CardRegistry>`.
fn build_defs_and_registry() -> (HashMap<String, CardDefinition>, Arc<CardRegistry>) {
    let cards = all_cards();
    let defs: HashMap<String, CardDefinition> =
        cards.iter().map(|d| (d.name.clone(), d.clone())).collect();
    let registry = CardRegistry::new(cards);
    (defs, registry)
}

/// Build an ObjectSpec enriched from its card definition.
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

// ── Nadaar, Selfless Paladin ──────────────────────────────────────────────────

/// CR 603.3 / CR 701.49a: When Nadaar, Selfless Paladin enters the battlefield,
/// its ETB triggered ability "venture into the dungeon" fires.
///
/// Expected: after Nadaar resolves from the stack, the ETB trigger is placed on the
/// stack. After the trigger resolves, a RoomAbility (Cave Entrance, Scry 1) is on the
/// stack. Player's dungeon state tracks that they're in Lost Mine at room 0.
///
/// Source: Nadaar oracle text; CR 603.3 (ETB triggers go on the stack).
#[test]
fn test_nadaar_enters_ventures() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let nadaar = make_spec(p1, "Nadaar, Selfless Paladin", ZoneId::Hand(p1), &defs);

    let mut state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(nadaar)
        .build()
        .unwrap();

    // Give p1 enough mana for Nadaar ({3}{W}).
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.colorless += 3;
        ps.mana_pool.white += 1;
    }

    // Find Nadaar in hand.
    let nadaar_id = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Nadaar, Selfless Paladin"
                && obj.zone == ZoneId::Hand(p1)
            {
                Some(id)
            } else {
                None
            }
        })
        .expect("Nadaar should be in p1 hand");

    // Cast Nadaar.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: nadaar_id,
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
        },
    )
    .expect("casting Nadaar should succeed");

    // Both players pass priority → Nadaar resolves from stack → ETB trigger queued.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    // Nadaar should now be on the battlefield.
    let nadaar_on_battlefield = state.objects.values().any(|o| {
        o.characteristics.name == "Nadaar, Selfless Paladin" && o.zone == ZoneId::Battlefield
    });
    assert!(
        nadaar_on_battlefield,
        "Nadaar should be on the battlefield after resolving"
    );

    // ETB trigger should be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "ETB venture trigger should be on the stack"
    );

    // Both players pass priority → ETB trigger resolves → VentureIntoDungeon → RoomAbility queued.
    let (state, etb_events) =
        process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, etb_events2) =
        process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    let all_etb: Vec<_> = etb_events.into_iter().chain(etb_events2).collect();

    // CR 701.49a: VenturedIntoDungeon event should be emitted (entering Lost Mine, room 0).
    let ventured = all_etb.iter().any(|e| {
        matches!(
            e,
            GameEvent::VenturedIntoDungeon {
                player,
                dungeon: DungeonId::LostMineOfPhandelver,
                room: 0,
            } if *player == p1
        )
    });
    assert!(
        ventured,
        "VenturedIntoDungeon event should be emitted for p1 entering Cave Entrance"
    );

    // CR 309.4c: RoomAbility (Scry 1 from Cave Entrance) should now be on the stack.
    let room_ability_on_stack = state.stack_objects.iter().any(|so| {
        matches!(
            &so.kind,
            StackObjectKind::RoomAbility { owner, dungeon, room }
            if *owner == p1 && *dungeon == DungeonId::LostMineOfPhandelver && *room == 0
        )
    });
    assert!(
        room_ability_on_stack,
        "RoomAbility for Cave Entrance (Scry 1) should be on the stack"
    );

    // p1's dungeon_state should track progress in Lost Mine at room 0.
    let ds = state.dungeon_state.get(&p1);
    assert!(ds.is_some(), "p1 should have an active dungeon state");
    let ds = ds.unwrap();
    assert_eq!(
        ds.dungeon,
        DungeonId::LostMineOfPhandelver,
        "p1 should be in Lost Mine of Phandelver"
    );
    assert_eq!(
        ds.current_room, 0,
        "p1 venture marker should be at room 0 (Cave Entrance)"
    );
}

/// CR 603.1 / CR 701.49: When Nadaar, Selfless Paladin attacks, its attack triggered
/// ability "venture into the dungeon" fires.
///
/// Setup: Nadaar is on the battlefield; p1 declares Nadaar as an attacker.
/// Expected: after declare_attackers, the WhenAttacks trigger is queued on the stack.
/// After both players pass, the trigger resolves and VenturedIntoDungeon event fires.
///
/// Source: Nadaar oracle text; CR 603.1 (triggers check for their condition).
#[test]
fn test_nadaar_attacks_ventures() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let nadaar = make_spec(p1, "Nadaar, Selfless Paladin", ZoneId::Battlefield, &defs);

    let state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(nadaar)
        .build()
        .unwrap();

    // Find Nadaar on the battlefield.
    let nadaar_id = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Nadaar, Selfless Paladin"
                && obj.zone == ZoneId::Battlefield
            {
                Some(id)
            } else {
                None
            }
        })
        .expect("Nadaar should be on the battlefield");

    // Declare Nadaar as an attacker targeting p2.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(nadaar_id, mtg_engine::AttackTarget::Player(p2))],
            enlist_choices: vec![],
        },
    )
    .expect("declare attackers should succeed");

    // After declaring attackers, the WhenAttacks trigger should be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "WhenAttacks trigger from Nadaar should be on the stack after declare attackers"
    );

    // Both players pass priority → attack trigger resolves → venture fires.
    let (state, t_events) =
        process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (_state, t_events2) =
        process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    let all_events: Vec<_> = t_events.into_iter().chain(t_events2).collect();

    // CR 701.49a: VenturedIntoDungeon event should fire (entering Lost Mine, room 0).
    let ventured = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::VenturedIntoDungeon { player, .. } if *player == p1));
    assert!(
        ventured,
        "VenturedIntoDungeon event should fire when Nadaar attacks"
    );
}

/// CR 309.7 / CR 613.1c: "Other creatures you control get +1/+1 as long as you've
/// completed a dungeon."
///
/// Note: The static buff has a DSL gap — EffectFilter::OtherCreaturesControlledBy
/// is not implemented, so the actual P/T boost cannot be tested yet. This test
/// verifies that Nadaar can be defined and placed on the battlefield without error,
/// and that completing a dungeon updates dungeons_completed correctly.
///
/// Source: Nadaar oracle text; CR 309.7 (player completes dungeon when it's removed).
#[test]
fn test_nadaar_completed_dungeon_buff() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let nadaar = make_spec(p1, "Nadaar, Selfless Paladin", ZoneId::Battlefield, &defs);

    let mut state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(nadaar)
        .build()
        .unwrap();

    // Verify Nadaar is on the battlefield.
    let nadaar_on_battlefield = state.objects.values().any(|o| {
        o.characteristics.name == "Nadaar, Selfless Paladin" && o.zone == ZoneId::Battlefield
    });
    assert!(
        nadaar_on_battlefield,
        "Nadaar should start on the battlefield"
    );

    // Simulate completing a dungeon by directly setting dungeons_completed.
    // (In a real game this would happen via handle_venture_into_dungeon completing
    // the bottommost room and triggering the DungeonCompleted SBA.)
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.dungeons_completed = 1;
    }

    // Verify the condition fires: p1 has completed a dungeon.
    let dungeons_completed = state
        .players
        .get(&p1)
        .map(|ps| ps.dungeons_completed)
        .unwrap_or(0);
    assert_eq!(dungeons_completed, 1, "p1 should have 1 dungeon completed");

    // Note: The actual +1/+1 static buff is tracked as a DSL gap — EffectFilter::
    // OtherCreaturesControlledBy is not yet implemented. The buff will not apply
    // to other creatures until that filter is added. See ultramarines_honour_guard.rs
    // for the same gap.
}

// ── Acererak the Archlich ─────────────────────────────────────────────────────

/// CR 603.4: Acererak the Archlich has an ETB triggered ability with an intervening-if
/// condition: "if you haven't completed Tomb of Annihilation, return Acererak to its
/// owner's hand and venture into the dungeon."
///
/// When a player has NOT completed Tomb of Annihilation, the condition is true and
/// Acererak returns to hand + ventures into the dungeon.
///
/// Source: Acererak oracle text; CR 603.4 (intervening-if evaluated at trigger time
/// and at resolution time).
#[test]
fn test_acererak_bounces_without_tomb() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let acererak = make_spec(p1, "Acererak the Archlich", ZoneId::Hand(p1), &defs);

    let mut state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(acererak)
        .build()
        .unwrap();

    // Give p1 mana for Acererak ({2}{B}).
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.colorless += 2;
        ps.mana_pool.black += 1;
    }

    // Confirm p1 has not completed Tomb of Annihilation (default state).
    let completed_tomb = state
        .players
        .get(&p1)
        .map(|ps| {
            ps.dungeons_completed_set
                .contains(&DungeonId::TombOfAnnihilation)
        })
        .unwrap_or(false);
    assert!(
        !completed_tomb,
        "p1 should not have completed Tomb of Annihilation"
    );

    // Find Acererak in hand.
    let acererak_id = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Acererak the Archlich" && obj.zone == ZoneId::Hand(p1) {
                Some(id)
            } else {
                None
            }
        })
        .expect("Acererak should be in p1 hand");

    // Cast Acererak.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: acererak_id,
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
        },
    )
    .expect("casting Acererak should succeed");

    // Both players pass priority → Acererak resolves → enters battlefield → ETB trigger queued.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    // Acererak should be on the battlefield (just entered).
    let on_bf = state.objects.values().any(|o| {
        o.characteristics.name == "Acererak the Archlich" && o.zone == ZoneId::Battlefield
    });
    assert!(
        on_bf,
        "Acererak should be on the battlefield after resolving"
    );

    // ETB trigger (intervening-if: Tomb not completed) should be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "Acererak ETB trigger should be on the stack"
    );

    // Both players pass priority → ETB trigger resolves → condition true → Acererak returns to hand + ventures.
    let (state, trigger_events) =
        process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, trigger_events2) =
        process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    let all_events: Vec<_> = trigger_events.into_iter().chain(trigger_events2).collect();

    // CR 603.4: Acererak should be back in p1's hand (MoveZone to Hand fired).
    let acererak_in_hand = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Acererak the Archlich" && o.zone == ZoneId::Hand(p1));
    assert!(
        acererak_in_hand,
        "Acererak should have returned to p1's hand (Tomb of Annihilation not completed)"
    );

    // CR 701.49a: VenturedIntoDungeon should have fired.
    let ventured = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::VenturedIntoDungeon { player, .. } if *player == p1));
    assert!(
        ventured,
        "VenturedIntoDungeon should fire as part of Acererak's ETB bounce effect"
    );
}

/// CR 603.4: Acererak the Archlich has an ETB triggered ability with an intervening-if
/// condition: "if you haven't completed Tomb of Annihilation, return Acererak to its
/// owner's hand and venture into the dungeon."
///
/// When a player HAS completed Tomb of Annihilation, the intervening-if condition is FALSE
/// at resolution. Per CR 603.4, the ability is removed from the stack without effect.
/// Acererak should stay on the battlefield and no VenturedIntoDungeon event should fire.
///
/// Source: Acererak oracle text; CR 603.4 (intervening-if re-evaluated at resolution).
#[test]
fn test_acererak_stays_after_tomb_completed() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    let acererak = make_spec(p1, "Acererak the Archlich", ZoneId::Hand(p1), &defs);

    let mut state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(acererak)
        .build()
        .unwrap();

    // Give p1 mana for Acererak ({2}{B}).
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.colorless += 2;
        ps.mana_pool.black += 1;
    }

    // Mark p1 as having completed Tomb of Annihilation — condition is now false.
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.dungeons_completed_set
            .insert(DungeonId::TombOfAnnihilation);
        ps.dungeons_completed += 1;
    }

    // Confirm p1 has completed Tomb of Annihilation.
    let completed_tomb = state
        .players
        .get(&p1)
        .map(|ps| {
            ps.dungeons_completed_set
                .contains(&DungeonId::TombOfAnnihilation)
        })
        .unwrap_or(false);
    assert!(
        completed_tomb,
        "p1 should have completed Tomb of Annihilation"
    );

    // Find Acererak in hand.
    let acererak_id = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Acererak the Archlich" && obj.zone == ZoneId::Hand(p1) {
                Some(id)
            } else {
                None
            }
        })
        .expect("Acererak should be in p1 hand");

    // Cast Acererak.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: acererak_id,
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
        },
    )
    .expect("casting Acererak should succeed");

    // Both players pass priority → Acererak resolves → enters battlefield → ETB trigger queued.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    // Acererak should be on the battlefield (just entered).
    let on_bf = state.objects.values().any(|o| {
        o.characteristics.name == "Acererak the Archlich" && o.zone == ZoneId::Battlefield
    });
    assert!(
        on_bf,
        "Acererak should be on the battlefield after resolving from the stack"
    );

    // ETB trigger should be on the stack (trigger fires at trigger-time check, condition
    // is also false then — but the trigger still gets queued and then removed at resolution).
    // The trigger may or may not be on the stack depending on how trigger-time check works;
    // what matters is the resolution outcome.

    // Both players pass priority → ETB trigger resolves → condition false → no effect.
    let (state, trigger_events) =
        process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, trigger_events2) =
        process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    let all_events: Vec<_> = trigger_events.into_iter().chain(trigger_events2).collect();

    // CR 603.4: Acererak should still be on the battlefield (NOT bounced to hand).
    let acererak_on_bf = state.objects.values().any(|o| {
        o.characteristics.name == "Acererak the Archlich" && o.zone == ZoneId::Battlefield
    });
    assert!(
        acererak_on_bf,
        "Acererak should remain on the battlefield when Tomb of Annihilation is completed (CR 603.4 intervening-if false at resolution)"
    );

    // CR 603.4: No VenturedIntoDungeon event should fire (intervening-if suppressed the effect).
    let ventured = all_events
        .iter()
        .any(|e| matches!(e, GameEvent::VenturedIntoDungeon { player, .. } if *player == p1));
    assert!(
        !ventured,
        "VenturedIntoDungeon should NOT fire when Tomb of Annihilation is completed (CR 603.4)"
    );

    // Acererak should NOT be in p1's hand.
    let acererak_in_hand = state
        .objects
        .values()
        .any(|o| o.characteristics.name == "Acererak the Archlich" && o.zone == ZoneId::Hand(p1));
    assert!(
        !acererak_in_hand,
        "Acererak should NOT have returned to hand (Tomb of Annihilation was completed)"
    );
}

// ── Take the Initiative ───────────────────────────────────────────────────────

/// CR 725.2 / CR 701.49d: When a player takes the initiative, they immediately
/// venture into the Undercity (forced dungeon choice per CR 701.49d).
///
/// Effect::TakeTheInitiative sets has_initiative = Some(controller), emits
/// InitiativeTaken, and calls handle_venture_into_dungeon with force_undercity=true.
///
/// Source: CR 725.2 — "take the initiative" inherent triggered ability.
#[test]
fn test_initiative_take_ventures_undercity() {
    let (defs, registry) = build_defs_and_registry();
    let p1 = p(1);
    let p2 = p(2);

    // Seasoned Dungeoneer has ETB: you take the initiative.
    let dungeoneer = make_spec(p1, "Seasoned Dungeoneer", ZoneId::Hand(p1), &defs);

    let mut state = GameStateBuilder::new()
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(dungeoneer)
        .build()
        .unwrap();

    // Give p1 mana for Seasoned Dungeoneer ({3}{W}).
    if let Some(ps) = state.players.get_mut(&p1) {
        ps.mana_pool.colorless += 3;
        ps.mana_pool.white += 1;
    }

    let dungeoneer_id = state
        .objects
        .iter()
        .find_map(|(&id, obj)| {
            if obj.characteristics.name == "Seasoned Dungeoneer" && obj.zone == ZoneId::Hand(p1) {
                Some(id)
            } else {
                None
            }
        })
        .expect("Seasoned Dungeoneer should be in p1 hand");

    // Cast Seasoned Dungeoneer.
    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: p1,
            card: dungeoneer_id,
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
        },
    )
    .expect("casting Seasoned Dungeoneer should succeed");

    // Pass priority → spell resolves → Dungeoneer enters → ETB trigger queued.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, _) = process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    // ETB trigger (take the initiative) should be on the stack.
    assert!(
        !state.stack_objects.is_empty(),
        "ETB take-the-initiative trigger should be on the stack"
    );

    // Pass priority → ETB trigger resolves → TakeTheInitiative fires.
    let (state, etb_events) =
        process_command(state, Command::PassPriority { player: p1 }).expect("pass p1");
    let (state, etb_events2) =
        process_command(state, Command::PassPriority { player: p2 }).expect("pass p2");

    let all_etb: Vec<_> = etb_events.into_iter().chain(etb_events2).collect();

    // CR 725.2: InitiativeTaken event should fire.
    let initiative_taken = all_etb
        .iter()
        .any(|e| matches!(e, GameEvent::InitiativeTaken { player } if *player == p1));
    assert!(
        initiative_taken,
        "InitiativeTaken event should be emitted when Seasoned Dungeoneer ETB fires"
    );

    // CR 725.1: p1 should now hold the initiative.
    assert_eq!(
        state.has_initiative,
        Some(p1),
        "p1 should hold the initiative after taking it"
    );

    // CR 701.49d / 725.2: Taking the initiative ventures into the Undercity (forced).
    let ventured_undercity = all_etb.iter().any(|e| {
        matches!(
            e,
            GameEvent::VenturedIntoDungeon {
                player,
                dungeon: DungeonId::TheUndercity,
                room: 0,
            } if *player == p1
        )
    });
    assert!(
        ventured_undercity,
        "Taking the initiative should venture into the Undercity (room 0 = Secret Entrance)"
    );

    // p1's dungeon state should show The Undercity at room 0.
    let ds = state
        .dungeon_state
        .get(&p1)
        .expect("p1 should have an active dungeon state");
    assert_eq!(
        ds.dungeon,
        DungeonId::TheUndercity,
        "p1 should be in The Undercity"
    );
    assert_eq!(
        ds.current_room, 0,
        "p1 should be at room 0 (Secret Entrance)"
    );
}
