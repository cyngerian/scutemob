//! PB-EF3: attack-trigger target fidelity (EF-W-MISS-10) + defending-player target
//! primitives (EF-W-MISS-4).
//!
//! Part A (EF-W-MISS-10): `enrich_spec_from_def` used to hardcode `targets: vec![]` for
//! every `AbilityDefinition::Triggered` conversion, silently dropping a card's declared
//! CR 601.2c target requirement; a registry-fallback bug additionally raw-indexed the
//! wrong ability on multi-ability cards. Fixed by A1 (forward `targets` in every enrich
//! block) + A2 (make the runtime `triggered_abilities` vec authoritative for
//! `PendingTriggerKind::Normal`, never falling through to the def raw-index).
//!
//! Part B (EF-W-MISS-4): adds `EffectTarget::AttackTarget` (the player or planeswalker
//! the triggering attacker is/was attacking, CR 508.4/506.4c) and
//! `PlayerTarget::DefendingPlayer` (the defending player alone, CR 508.4), with the
//! defending player captured at attack-trigger dispatch (CR 113.7a) rather than looked
//! up lazily at resolution.
//!
//! CR Rules covered: 508.1m, 508.4, 506.4c, 601.2c, 603.3d, 113.7a.

use std::collections::HashMap;

use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AttackTarget, CardDefinition, CardId,
    CardRegistry, CardType, Command, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder,
    ObjectId, ObjectSpec, PlayerId, PlayerTarget, Step, TriggerEvent, TriggeredAbilityDef, ZoneId,
};

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
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn life(state: &GameState, player: PlayerId) -> i32 {
    state.players().get(&player).unwrap().life_total
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

/// Build a `HashMap<String, CardDefinition>` keyed by card name, as required by
/// `enrich_spec_from_def` (the production path that lowers a CardDef's declared
/// `AbilityDefinition::Triggered` into a runtime `TriggeredAbilityDef`).
fn make_defs(defs: Vec<CardDefinition>) -> HashMap<String, CardDefinition> {
    defs.into_iter().map(|d| (d.name.clone(), d)).collect()
}

/// Look up a real card definition from the compiled registry by exact name.
fn find_card_def(name: &str) -> CardDefinition {
    all_cards()
        .iter()
        .find(|d| d.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("card '{}' must be in the card registry", name))
}

// ── Part A: attack-trigger target fidelity (EF-W-MISS-10) ────────────────────

/// CR 508.1m / CR 601.2c — PB-EF3 A: Ojutai, Soul of Winter's "tap target nonland
/// permanent an opponent controls" declared target is forwarded into the runtime
/// trigger and correctly auto-selected (CR 603.3d), instead of being silently dropped.
///
/// Setup: P1 controls Ojutai (real card def, a Dragon). P2 controls a single nonland
/// permanent (an artifact). P1 attacks P2 with Ojutai.
///
/// Assert: after the trigger resolves, P2's artifact is tapped AND carries
/// `skip_untap_steps == 1` (CR 502.3 — doesn't untap during its controller's next
/// untap step).
#[test]
fn test_attack_trigger_forwards_declared_target() {
    let p1 = p(1);
    let p2 = p(2);

    let ojutai_def = find_card_def("Ojutai, Soul of Winter");
    let defs = make_defs(vec![ojutai_def.clone()]);

    let ojutai_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Ojutai, Soul of Winter")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(CardId("ojutai-soul-of-winter".to_string())),
        &defs,
    );

    let opponent_artifact = ObjectSpec::card(p2, "Opponent Artifact")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![ojutai_def]))
        .object(ojutai_spec)
        .object(opponent_artifact)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let ojutai_id = find_obj(&state, "Ojutai, Soul of Winter");
    let artifact_id = find_obj(&state, "Opponent Artifact");

    // CR 508.1m: declaring Ojutai as an attacker fires "whenever a Dragon you control
    // attacks" immediately (before blockers). handle_declare_attackers flushes the
    // trigger to the stack synchronously.
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(ojutai_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers with Ojutai should succeed");

    // Both players pass priority — the tap trigger resolves.
    let (state, _) = pass_all(state, &[p1, p2]);

    let artifact = state.objects().get(&artifact_id).unwrap();
    assert!(
        artifact.status.tapped,
        "CR 508.1m/601.2c: Ojutai's attack trigger should have tapped P2's nonland \
         permanent (target was forwarded and auto-selected, not dropped)"
    );
    assert_eq!(
        artifact.skip_untap_steps, 1,
        "CR 502.3: the tapped permanent should not untap during its controller's next \
         untap step"
    );
}

/// CR 601.2c / CR 603.3d — PB-EF3 A (decoy): pins the EF-W-MISS-10 regression and
/// discriminates target *identity*, not just "something got tapped".
///
/// Verified non-vacuous (fix-phase experiment, reverted before commit): reverting A1's
/// `targets: targets.clone()` back to `targets: vec![]` in the
/// `WheneverCreatureYouControlAttacks` enrich block makes this test (and
/// `test_attack_trigger_forwards_declared_target`) fail — nothing gets tapped.
///
/// Note on A2 in isolation: for a card with exactly one runtime-registered triggered
/// ability (Ojutai's case — `characteristics.triggered_abilities` has a single entry, so
/// `trigger.ability_index` trivially indexes it correctly), A2's raw-index fallback path
/// is only ever *reached* when the runtime lookup already found empty targets — i.e. only
/// when A1 is also broken for that ability. A2's fallback-removal is independently proven
/// load-bearing by a *different* real-card regression this PB found and fixed: Throat
/// Slitter, Elder Deep Fiend, and WheneverRingTemptsYou-style triggers push a raw
/// `def.abilities` index with `PendingTriggerKind::Normal` (mirroring an existing
/// `CardDefETB` pattern but mis-tagged); reclassifying them to `CardDefETB` is what makes
/// A2's strict Normal-kind guard correct rather than a regression. Confirmed by
/// experiment: reverting that reclassification alone (kind back to `Normal`, A1/A2 left
/// fixed) fails `pbd_damaged_player_filter::test_throat_slitter_end_to_end_precision_fix`.
///
/// Setup: same as the positive case, but P2 also controls a land (an illegal target —
/// `non_land: true` in the filter) to prove the *correct* permanent was selected, not
/// merely *some* permanent.
///
/// Assert: P2's artifact (the only legal nonland permanent) is tapped; P2's land is not.
#[test]
fn test_attack_trigger_target_not_dropped_decoy() {
    let p1 = p(1);
    let p2 = p(2);

    let ojutai_def = find_card_def("Ojutai, Soul of Winter");
    let defs = make_defs(vec![ojutai_def.clone()]);

    let ojutai_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Ojutai, Soul of Winter")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(CardId("ojutai-soul-of-winter".to_string())),
        &defs,
    );

    let opponent_artifact = ObjectSpec::card(p2, "Opponent Artifact")
        .with_types(vec![CardType::Artifact])
        .in_zone(ZoneId::Battlefield);
    // Decoy: a land is NOT a legal target (non_land filter) — it must stay untapped.
    let opponent_land = ObjectSpec::card(p2, "Opponent Land")
        .with_types(vec![CardType::Land])
        .in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![ojutai_def]))
        .object(ojutai_spec)
        .object(opponent_artifact)
        .object(opponent_land)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let ojutai_id = find_obj(&state, "Ojutai, Soul of Winter");
    let artifact_id = find_obj(&state, "Opponent Artifact");
    let land_id = find_obj(&state, "Opponent Land");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(ojutai_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers with Ojutai should succeed");

    let (state, _) = pass_all(state, &[p1, p2]);

    assert!(
        state.objects().get(&artifact_id).unwrap().status.tapped,
        "EF-W-MISS-10 regression pin: the declared target (nonland permanent) should be \
         tapped. If A1 or A2 regresses, Ojutai's Triggered ability (def index 2, behind \
         Flying/Vigilance) would be raw-indexed to Flying and the target silently dropped."
    );
    assert!(
        !state.objects().get(&land_id).unwrap().status.tapped,
        "CR 601.2c: the land is not a legal target (non_land filter) and must not be tapped"
    );
}

// ── Part B: defending-player target primitives (EF-W-MISS-4) ─────────────────

/// CR 508.4 — PB-EF3 B: Hellrider (real card def) deals its 1 damage to the specific
/// defending player, not to every opponent.
///
/// Setup: 4-player game. P1 controls Hellrider and attacks P2 alone.
///
/// Assert: P2 loses 1 life. P3 and P4 (not attacked) are unaffected — this is the
/// decoy against an `EachOpponent`-style substitution, which would be wrong in a
/// 4-player game (CR 508.4: an attacker attacks exactly one defending player).
#[test]
fn test_hellrider_damages_defending_player_4p() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let hellrider_def = find_card_def("Hellrider");
    let defs = make_defs(vec![hellrider_def.clone()]);

    let hellrider_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Hellrider")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(CardId("hellrider".to_string())),
        &defs,
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![hellrider_def]))
        .object(hellrider_spec)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let hellrider_id = find_obj(&state, "Hellrider");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(hellrider_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers with Hellrider should succeed");

    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    assert_eq!(
        life(&state, p2),
        39,
        "CR 508.4: Hellrider should deal 1 damage to P2, the defending player"
    );
    assert_eq!(
        life(&state, p3),
        40,
        "CR 508.4 decoy: P3 (not attacked) must be unaffected"
    );
    assert_eq!(
        life(&state, p4),
        40,
        "CR 508.4 decoy: P4 (not attacked) must be unaffected"
    );
}

/// CR 508.4 / CR 506.4c — PB-EF3 B: Hellrider deals its damage to an attacked
/// planeswalker directly, not to the planeswalker's controller's life total.
///
/// Setup: 3-player game. P1 controls Hellrider and attacks a planeswalker controlled
/// by P3 (5 loyalty).
///
/// Assert: the planeswalker loses 1 loyalty (4 remaining). P3's life total is
/// unaffected (damage to a planeswalker is not damage to its controller).
#[test]
fn test_hellrider_damages_attacked_planeswalker() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let hellrider_def = find_card_def("Hellrider");
    let defs = make_defs(vec![hellrider_def.clone()]);

    let hellrider_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Hellrider")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(CardId("hellrider".to_string())),
        &defs,
    );

    let planeswalker = ObjectSpec::planeswalker(p3, "Test Planeswalker", 5)
        .with_counter(mtg_engine::CounterType::Loyalty, 5);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(CardRegistry::new(vec![hellrider_def]))
        .object(hellrider_spec)
        .object(planeswalker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let hellrider_id = find_obj(&state, "Hellrider");
    let pw_id = find_obj(&state, "Test Planeswalker");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(hellrider_id, AttackTarget::Planeswalker(pw_id))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers targeting the planeswalker should succeed");

    let (state, _) = pass_all(state, &[p1, p2, p3]);

    let pw = state.objects().get(&pw_id).unwrap();
    let loyalty = pw
        .counters
        .get(&mtg_engine::CounterType::Loyalty)
        .copied()
        .unwrap_or(0);
    assert_eq!(
        loyalty, 4,
        "CR 508.4/506.4c: Hellrider's damage should hit the attacked planeswalker \
         directly (5 - 1 = 4 loyalty), not P3's life total"
    );
    assert_eq!(
        life(&state, p3),
        40,
        "CR 306.8: damage to a planeswalker is not damage to its controller"
    );
}

/// CR 508.4 — PB-EF3 B: `PlayerTarget::DefendingPlayer` (Brutal-Hordechief-style
/// "defending player loses N life") scopes to the specific defending player in a
/// multiplayer game. Brutal Hordechief itself is not shipped (its second ability is
/// inexpressible), so the ability is authored inline on a test creature.
///
/// Setup: 4-player game. P1 controls a test creature with
/// `LoseLife { player: DefendingPlayer, amount: 1 }` on `WheneverCreatureYouControlAttacks`.
/// P1 attacks P2 alone.
///
/// Assert: P2 loses 1 life. P3 and P4 are unaffected (decoy against `EachOpponent`).
#[test]
fn test_defending_player_target_multiplayer() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let attacker = ObjectSpec::creature(p1, "BH Test Creature", 3, 3).with_triggered_ability(
        TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
            intervening_if: None,
            targets: vec![],
            description: "test: defending player loses 1 life".to_string(),
            effect: Some(Effect::LoseLife {
                player: PlayerTarget::DefendingPlayer,
                amount: EffectAmount::Fixed(1),
            }),
        },
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "BH Test Creature");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2, p3, p4]);

    assert_eq!(
        life(&state, p2),
        39,
        "CR 508.4: PlayerTarget::DefendingPlayer should scope to P2, the defending player"
    );
    assert_eq!(life(&state, p3), 40, "decoy: P3 must be unaffected");
    assert_eq!(life(&state, p4), 40, "decoy: P4 must be unaffected");
}

/// CR 508.1m / EF-W-MISS-4 — PB-EF3 B: Raid Bombardment's `max_power` filter (real
/// card def) is checked against the SPECIFIC attacking creature, and the resulting
/// damage is routed to THAT creature's specific attack target via
/// `EffectTarget::AttackTarget` — not to every attacked player uniformly.
///
/// Setup: 3-player game. P1 controls Raid Bombardment, a power-2 attacker (attacks P2),
/// and a power-3 attacker (attacks P3), declared in the same combat.
///
/// Assert: P2 (attacked by the qualifying power-2 creature) loses 1 life. P3 (attacked
/// by the non-qualifying power-3 creature) is unaffected — proving the filter is
/// evaluated per-attacker, not "any attacker with power <= 2 anywhere unlocks damage to
/// whoever is attacked."
#[test]
fn test_raid_bombardment_power_filter() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let raid_bombardment_def = find_card_def("Raid Bombardment");
    let defs = make_defs(vec![raid_bombardment_def.clone()]);

    let enchantment_spec = enrich_spec_from_def(
        ObjectSpec::card(p1, "Raid Bombardment")
            .in_zone(ZoneId::Battlefield)
            .with_card_id(CardId("raid-bombardment".to_string())),
        &defs,
    );

    let power_two_attacker = ObjectSpec::creature(p1, "Power Two Attacker", 2, 2);
    let power_three_attacker = ObjectSpec::creature(p1, "Power Three Attacker", 3, 3);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(CardRegistry::new(vec![raid_bombardment_def]))
        .object(enchantment_spec)
        .object(power_two_attacker)
        .object(power_three_attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let power_two_id = find_obj(&state, "Power Two Attacker");
    let power_three_id = find_obj(&state, "Power Three Attacker");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![
                (power_two_id, AttackTarget::Player(p2)),
                (power_three_id, AttackTarget::Player(p3)),
            ],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    let (state, _) = pass_all(state, &[p1, p2, p3]);

    assert_eq!(
        life(&state, p2),
        39,
        "CR 508.1m: the power-2 attacker qualifies (power <= 2); P2 (its attack target) \
         should lose 1 life"
    );
    assert_eq!(
        life(&state, p3),
        40,
        "CR 508.1m: the power-3 attacker does NOT qualify (power > 2); P3 (its attack \
         target) should be unaffected"
    );
}

/// CR 113.7a / CR 508.4 — PB-EF3 B: the defending player is captured at attack-trigger
/// dispatch time and survives the attacker later leaving combat, proving
/// capture-at-dispatch is necessary (a lazy re-derivation from `state.combat.attackers`
/// at resolution time would find nothing once the attacker's entry is gone).
///
/// Setup: 3-player game. P1's attacker (inline `DefendingPlayer` ability) attacks P2.
/// After the attack is declared (and the trigger is already on the stack, capturing
/// `defending_player_id = P2`), the attacker's entry is removed from
/// `state.combat.attackers` directly (simulating the attacker having left combat,
/// CR 506.4) before the trigger resolves.
///
/// Assert: P2 still loses 1 life — the captured value, not a live lookup, drove the
/// effect.
#[test]
fn test_defending_player_captured_survives_attacker_removal() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let attacker = ObjectSpec::creature(p1, "Removed Attacker", 2, 2).with_triggered_ability(
        TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            trigger_on: TriggerEvent::AnyCreatureYouControlAttacks,
            intervening_if: None,
            targets: vec![],
            description: "test: defending player loses 1 life (capture-at-dispatch)".to_string(),
            effect: Some(Effect::LoseLife {
                player: PlayerTarget::DefendingPlayer,
                amount: EffectAmount::Fixed(1),
            }),
        },
    );

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(CardRegistry::new(vec![]))
        .object(attacker)
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_obj(&state, "Removed Attacker");

    // CR 508.1m: declaring the attack fires the trigger and flushes it to the stack
    // synchronously (handle_declare_attackers), capturing defending_player_id = p2
    // on the PendingTrigger before this function returns.
    let (mut state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");

    assert!(
        !state.stack_objects().is_empty(),
        "the attack trigger should already be on the stack after DeclareAttackers"
    );

    // Simulate the attacker leaving combat (CR 506.4) before the trigger resolves:
    // remove its entry from state.combat.attackers directly. A lazy re-derivation of
    // the defending player from this map would now find nothing for this attacker.
    if let Some(combat) = state.combat_mut() {
        combat.attackers.remove(&attacker_id);
    }

    let (state, _) = pass_all(state, &[p1, p2, p3]);

    assert_eq!(
        life(&state, p2),
        39,
        "CR 113.7a: the defending player was captured at dispatch time and must still \
         apply even though the attacker's combat.attackers entry was removed before \
         the trigger resolved"
    );
}
