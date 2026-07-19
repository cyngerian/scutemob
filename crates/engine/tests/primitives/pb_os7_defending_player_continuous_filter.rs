//! PB-OS7 (OOS-EF3-1): `EffectFilter::CreaturesControlledByDefendingPlayer`.
//!
//! CR 508.4 (defending player) / 611.2a (continuous effect locked in at resolution) /
//! 613.1d/613.4c (Layer 7c P/T modification) / 514.2 (until-end-of-turn cleanup) /
//! 704.5f (0-toughness SBA) / 205.3m (Dragon subtype). A DSL placeholder
//! `EffectFilter`, substituted at `Effect::ApplyContinuousEffect` execution time into
//! the pre-existing locked `EffectFilter::CreaturesControlledBy(pid)` using the
//! per-attacker captured `ctx.defending_player` (PB-EF3 machinery). `None =>` the
//! effect is skipped entirely — it must NEVER fall back to the controller (that would
//! debuff the caster's own creatures).
//!
//! Ships one new card: Silumgar, the Drifting Death ("Whenever a Dragon you control
//! attacks, creatures defending player controls get -1/-1 until end of turn.").
//! Ruling 2014-11-24: the affected set is relative to the SPECIFIC attacking Dragon —
//! each attacking Dragon triggers separately and scopes to its own defending player.
//!
//! Test pattern: attack triggers driven through the real `Command::DeclareAttackers`
//! path (mirrors `pb_os5_relative_attacker_count.rs`), reading layer-resolved P/T via
//! `calculate_characteristics` AFTER the trigger resolves — execution-probing, not
//! source-tracing (SR-34/36). One test (`test_os7_no_defending_player_applies_to_nothing`)
//! constructs the effect and an `EffectContext` directly (mirrors
//! `pb_os5_relative_attacker_count.rs::test_os5_scope_animosity_piledriver_any_controller`)
//! because there is no real-command path to a trigger with `ctx.defending_player == None`
//! — the guard is exercised at the `execute_effect` boundary instead.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    all_cards, calculate_characteristics, enrich_spec_from_def, process_command, AttackTarget,
    CardContinuousEffectDef, CardDefinition, Command, EffectDuration, EffectFilter, EffectLayer,
    GameEvent, GameState, GameStateBuilder, LayerModification, ObjectId, ObjectSpec, PlayerId,
    Step, SubType, ZoneId, HASH_SCHEMA_VERSION, PROTOCOL_VERSION,
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
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn power(state: &GameState, id: ObjectId) -> Option<i32> {
    calculate_characteristics(state, id).and_then(|c| c.power)
}

fn toughness(state: &GameState, id: ObjectId) -> Option<i32> {
    calculate_characteristics(state, id).and_then(|c| c.toughness)
}

fn is_on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects()
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn is_in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state.objects().values().any(|o| {
        o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner) && o.owner == owner
    })
}

fn load_defs() -> HashMap<String, CardDefinition> {
    all_cards()
        .iter()
        .map(|d| (d.name.clone(), d.clone()))
        .collect()
}

fn place_card(owner: PlayerId, name: &str, defs: &HashMap<String, CardDefinition>) -> ObjectSpec {
    enrich_spec_from_def(
        ObjectSpec::card(owner, name).in_zone(ZoneId::Battlefield),
        defs,
    )
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

/// Drain the stack fully (repeated `pass_all` rounds) — needed when a single
/// `DeclareAttackers` synthesizes multiple stack objects (one per attacking Dragon,
/// CR 508.1m).
fn drain_stack(mut state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(
            guard < 50,
            "drain_stack: stack did not empty after 50 rounds"
        );
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

fn declare_attackers(
    state: GameState,
    player: PlayerId,
    attackers: Vec<(ObjectId, AttackTarget)>,
) -> GameState {
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player,
            attackers,
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers should succeed");
    state
}

// ── Test 1: defending player's creatures debuffed; attacker's own untouched ────

/// CR 508.4/611.2a — Silumgar attacks alone (its "Home Dragon" sibling stays back,
/// proving the trigger fires per ATTACKING Dragon, not per Dragon-you-control): the
/// defending player's creature gets -1/-1; the attacking player's own creatures
/// (including the non-attacking Home Dragon) are completely unaffected.
#[test]
fn test_os7_defending_player_creatures_debuffed() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let silumgar = place_card(p1, "Silumgar, the Drifting Death", &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(mtg_engine::CardRegistry::new(all_cards()))
        .object(silumgar)
        .object(
            ObjectSpec::creature(p1, "Home Dragon", 4, 4)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .object(ObjectSpec::creature(p1, "P1 Own Creature", 2, 2))
        .object(ObjectSpec::creature(p2, "Defender Creature", 3, 3))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let silumgar_id = find_object(&state, "Silumgar, the Drifting Death");
    let home_dragon_id = find_object(&state, "Home Dragon");
    let own_creature_id = find_object(&state, "P1 Own Creature");
    let defender_id = find_object(&state, "Defender Creature");

    // Only Silumgar attacks; Home Dragon stays back.
    let state = declare_attackers(state, p1, vec![(silumgar_id, AttackTarget::Player(p2))]);
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert_eq!(
        power(&state, defender_id),
        Some(2),
        "CR 508.4/611.2a: the defending player's creature should get -1/-1 (3 base -> 2)"
    );
    assert_eq!(toughness(&state, defender_id), Some(2));

    assert_eq!(
        power(&state, home_dragon_id),
        Some(4),
        "the attacking player's OWN creatures must be completely unaffected -- the \
         filter never falls back to the controller"
    );
    assert_eq!(toughness(&state, home_dragon_id), Some(4));
    assert_eq!(
        power(&state, own_creature_id),
        Some(2),
        "attacker's own non-Dragon creature must also be unaffected"
    );
    assert_eq!(toughness(&state, own_creature_id), Some(2));
}

// ── Test 2: 4-player bystander decoy ────────────────────────────────────────────

/// CR 508.4 — 4-player: Silumgar (p1) attacks p2 only. p2's creature gets -1/-1;
/// p3's and p4's creatures (bystanders, not the defending player) are completely
/// untouched. The wedge is non-vacuous: p3 and p4 each control a 1-toughness
/// creature that must SURVIVE (if the filter wrongly matched them, SBA would kill it).
#[test]
fn test_os7_four_player_bystander_decoy() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = load_defs();

    let silumgar = place_card(p1, "Silumgar, the Drifting Death", &defs);

    let state = GameStateBuilder::four_player()
        .with_registry(mtg_engine::CardRegistry::new(all_cards()))
        .object(silumgar)
        .object(ObjectSpec::creature(p2, "Defender Creature", 3, 3))
        .object(ObjectSpec::creature(p3, "P3 Fragile", 1, 1))
        .object(ObjectSpec::creature(p4, "P4 Fragile", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let silumgar_id = find_object(&state, "Silumgar, the Drifting Death");
    let defender_id = find_object(&state, "Defender Creature");
    let p3_fragile_id = find_object(&state, "P3 Fragile");
    let p4_fragile_id = find_object(&state, "P4 Fragile");

    let state = declare_attackers(state, p1, vec![(silumgar_id, AttackTarget::Player(p2))]);
    let (state, _) = drain_stack(state, &[p1, p2, p3, p4]);

    assert_eq!(
        power(&state, defender_id),
        Some(2),
        "CR 508.4: p2 is the defending player -- its creature gets -1/-1"
    );
    assert_eq!(toughness(&state, defender_id), Some(2));

    assert_eq!(
        power(&state, p3_fragile_id),
        Some(1),
        "p3 is a bystander (not the defending player) -- power must be untouched"
    );
    assert_eq!(
        toughness(&state, p3_fragile_id),
        Some(1),
        "p3's creature must NOT take -1 toughness -- it must survive the SBA check \
         below at its full base toughness"
    );
    assert!(
        is_on_battlefield(&state, "P3 Fragile"),
        "p3's 1-toughness creature must survive -- it was never in the affected set"
    );

    assert_eq!(power(&state, p4_fragile_id), Some(1));
    assert_eq!(toughness(&state, p4_fragile_id), Some(1));
    assert!(
        is_on_battlefield(&state, "P4 Fragile"),
        "p4's 1-toughness creature must survive -- it was never in the affected set"
    );
}

// ── Test 3: two Dragons attack the SAME opponent -> stacks to -2/-2 ────────────

/// Silumgar ruling 2014-11-24 / CR 611.2a — two Dragons you control attack the SAME
/// defending player: two separate triggers fire, each stamping its OWN
/// `CreaturesControlledBy(sameB)` continuous effect, so the effects stack: -2/-2.
#[test]
fn test_os7_multi_attack_same_defender_stacks() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let silumgar = place_card(p1, "Silumgar, the Drifting Death", &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(mtg_engine::CardRegistry::new(all_cards()))
        .object(silumgar)
        .object(
            ObjectSpec::creature(p1, "Second Dragon", 4, 4)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .object(ObjectSpec::creature(p2, "Defender Creature", 4, 4))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let silumgar_id = find_object(&state, "Silumgar, the Drifting Death");
    let second_dragon_id = find_object(&state, "Second Dragon");
    let defender_id = find_object(&state, "Defender Creature");

    let state = declare_attackers(
        state,
        p1,
        vec![
            (silumgar_id, AttackTarget::Player(p2)),
            (second_dragon_id, AttackTarget::Player(p2)),
        ],
    );
    let (state, _) = drain_stack(state, &[p1, p2]);

    assert_eq!(
        power(&state, defender_id),
        Some(2),
        "Silumgar ruling 2014-11-24: two Dragons attacking the SAME opponent stack -- \
         that opponent's creatures should be at -2/-2 (4 base -> 2)"
    );
    assert_eq!(toughness(&state, defender_id), Some(2));
}

// ── Test 4: two Dragons attack DIFFERENT opponents -> each own scope ───────────

/// Silumgar ruling 2014-11-24 (direct encoding) — 4-player: Silumgar attacks p2,
/// Second Dragon attacks p3. Each defending player's creatures get -1/-1 in their
/// OWN scope (not summed across both); p4 (a third opponent, attacked by neither) is
/// completely untouched.
#[test]
fn test_os7_multi_attack_different_defenders_scoped() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = load_defs();

    let silumgar = place_card(p1, "Silumgar, the Drifting Death", &defs);

    let state = GameStateBuilder::four_player()
        .with_registry(mtg_engine::CardRegistry::new(all_cards()))
        .object(silumgar)
        .object(
            ObjectSpec::creature(p1, "Second Dragon", 4, 4)
                .with_subtypes(vec![SubType("Dragon".to_string())]),
        )
        .object(ObjectSpec::creature(p2, "Defender B Creature", 3, 3))
        .object(ObjectSpec::creature(p3, "Defender C Creature", 3, 3))
        .object(ObjectSpec::creature(p4, "Bystander D Creature", 3, 3))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let silumgar_id = find_object(&state, "Silumgar, the Drifting Death");
    let second_dragon_id = find_object(&state, "Second Dragon");
    let defender_b_id = find_object(&state, "Defender B Creature");
    let defender_c_id = find_object(&state, "Defender C Creature");
    let bystander_d_id = find_object(&state, "Bystander D Creature");

    let state = declare_attackers(
        state,
        p1,
        vec![
            (silumgar_id, AttackTarget::Player(p2)),
            (second_dragon_id, AttackTarget::Player(p3)),
        ],
    );
    let (state, _) = drain_stack(state, &[p1, p2, p3, p4]);

    assert_eq!(
        power(&state, defender_b_id),
        Some(2),
        "p2 (attacked by Silumgar) gets -1/-1 in its own scope, not -2/-2"
    );
    assert_eq!(toughness(&state, defender_b_id), Some(2));

    assert_eq!(
        power(&state, defender_c_id),
        Some(2),
        "p3 (attacked by Second Dragon) gets -1/-1 in its own scope, not -2/-2"
    );
    assert_eq!(toughness(&state, defender_c_id), Some(2));

    assert_eq!(
        power(&state, bystander_d_id),
        Some(3),
        "p4 was attacked by neither Dragon -- completely untouched"
    );
    assert_eq!(toughness(&state, bystander_d_id), Some(3));
}

// ── Test 5: until-end-of-turn expiry ────────────────────────────────────────────

/// CR 514.2 — after Silumgar's trigger resolves, the -1/-1 is active pre-cleanup.
/// After `expire_end_of_turn_effects` runs (simulating Cleanup), the continuous
/// effect is removed and the defending player's creature returns to its printed P/T.
#[test]
fn test_os7_until_end_of_turn_expiry() {
    let p1 = p(1);
    let p2 = p(2);
    let defs = load_defs();

    let silumgar = place_card(p1, "Silumgar, the Drifting Death", &defs);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(mtg_engine::CardRegistry::new(all_cards()))
        .object(silumgar)
        .object(ObjectSpec::creature(p2, "Defender Creature", 3, 3))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let silumgar_id = find_object(&state, "Silumgar, the Drifting Death");
    let defender_id = find_object(&state, "Defender Creature");

    let state = declare_attackers(state, p1, vec![(silumgar_id, AttackTarget::Player(p2))]);
    let (mut state, _) = drain_stack(state, &[p1, p2]);

    assert_eq!(
        power(&state, defender_id),
        Some(2),
        "pre-cleanup: the -1/-1 should be active"
    );
    assert!(
        state.continuous_effects().iter().any(|e| matches!(
            e.modification,
            LayerModification::ModifyBoth(-1)
        ) && matches!(
            e.duration,
            EffectDuration::UntilEndOfTurn
        )),
        "precondition: a ModifyBoth(-1) UntilEndOfTurn continuous effect must be present \
         before cleanup"
    );

    mtg_engine::rules::layers::expire_end_of_turn_effects(&mut state);

    assert_eq!(
        power(&state, defender_id),
        Some(3),
        "CR 514.2: the -1/-1 should expire at cleanup, restoring base power"
    );
    assert_eq!(toughness(&state, defender_id), Some(3));
    assert!(
        !state.continuous_effects().iter().any(|e| matches!(
            e.modification,
            LayerModification::ModifyBoth(-1)
        ) && matches!(
            e.duration,
            EffectDuration::UntilEndOfTurn
        )),
        "the ModifyBoth(-1) UntilEndOfTurn continuous effect must be removed after cleanup"
    );
}

// ── Test 6: 0-toughness SBA — defender dies, bystander survives ────────────────

/// CR 704.5f — 4-player: p2 (the defending player) controls a 1-toughness creature,
/// which dies to the -1/-1 as a state-based action. p3 (a bystander opponent) also
/// controls a 1-toughness creature, which is untouched by the filter and survives.
#[test]
fn test_os7_toughness_death_sba_defender_vs_bystander() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let defs = load_defs();

    let silumgar = place_card(p1, "Silumgar, the Drifting Death", &defs);

    let state = GameStateBuilder::four_player()
        .with_registry(mtg_engine::CardRegistry::new(all_cards()))
        .object(silumgar)
        .object(ObjectSpec::creature(p2, "Defender Fragile", 1, 1))
        .object(ObjectSpec::creature(p3, "Bystander Fragile", 1, 1))
        .active_player(p1)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let silumgar_id = find_object(&state, "Silumgar, the Drifting Death");

    let state = declare_attackers(state, p1, vec![(silumgar_id, AttackTarget::Player(p2))]);
    let (state, _) = drain_stack(state, &[p1, p2, p3, p4]);

    assert!(
        is_in_graveyard(&state, "Defender Fragile", p2),
        "CR 704.5f: the defending player's 1-toughness creature should die to the \
         -1/-1 as a state-based action"
    );
    assert!(
        !is_on_battlefield(&state, "Defender Fragile"),
        "Defender Fragile must no longer be on the battlefield"
    );

    assert!(
        is_on_battlefield(&state, "Bystander Fragile"),
        "the bystander's 1-toughness creature was never in the affected set -- it \
         must survive"
    );
    assert_eq!(
        toughness(&state, find_object(&state, "Bystander Fragile")),
        Some(1),
        "bystander's toughness must be untouched"
    );
}

// ── Test 7: no defending player -> applies to nothing (footgun guard) ──────────

/// CR 508.4 — the `None => return` skip path. Exercises `ApplyContinuousEffect` with
/// `CreaturesControlledByDefendingPlayer` directly under an `EffectContext` whose
/// `defending_player` is `None` (there is no real-command path to this state --
/// `ctx.defending_player` is always populated by a real attack trigger). Asserts NO
/// creature (including the resolving player's own) is debuffed, and that no
/// `ContinuousEffect` is even pushed onto state -- the most important correctness
/// point of this primitive: it must NEVER fall back to `ctx.controller`.
#[test]
fn test_os7_no_defending_player_applies_to_nothing() {
    let p1 = p(1);
    let p2 = p(2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .object(ObjectSpec::creature(p1, "Some Source", 2, 2))
        .object(ObjectSpec::creature(p1, "Controller Creature", 3, 3))
        .build()
        .unwrap();

    let source_id = find_object(&state, "Some Source");
    let controller_creature_id = find_object(&state, "Controller Creature");

    assert!(
        state.continuous_effects().is_empty(),
        "precondition: no continuous effects registered yet"
    );

    let effect = mtg_engine::Effect::ApplyContinuousEffect {
        effect_def: Box::new(CardContinuousEffectDef {
            layer: EffectLayer::PtModify,
            modification: LayerModification::ModifyBoth(-1),
            filter: EffectFilter::CreaturesControlledByDefendingPlayer,
            duration: EffectDuration::UntilEndOfTurn,
            condition: None,
        }),
    };
    // EffectContext::new defaults defending_player to None -- exactly the guard path.
    let mut ctx = EffectContext::new(p1, source_id, vec![]);
    assert_eq!(
        ctx.defending_player, None,
        "precondition: no defending player captured"
    );
    execute_effect(&mut state, &effect, &mut ctx);

    assert!(
        state.continuous_effects().is_empty(),
        "None => return: the effect must be skipped entirely -- no ContinuousEffect \
         should ever be pushed onto state"
    );
    assert_eq!(
        power(&state, controller_creature_id),
        Some(3),
        "the resolving player's OWN creature must NOT be debuffed -- the filter must \
         never fall back to ctx.controller (that would be the exact footgun this \
         primitive is designed to avoid)"
    );
    assert_eq!(toughness(&state, controller_creature_id), Some(3));
}

// ── Test 8: registration smoke test ─────────────────────────────────────────────

/// `all_cards()` contains the new card def, and it loads without panicking.
#[test]
fn test_os7_card_registered() {
    let all = all_cards();
    assert!(
        all.iter().any(|d| d.name == "Silumgar, the Drifting Death"),
        "'Silumgar, the Drifting Death' should be present in all_cards()"
    );
}

// ── Test 9: wire sentinels ───────────────────────────────────────────────────────

/// PB-OS7 bumped HASH_SCHEMA_VERSION 58 -> 59 (a single new `EffectFilter` variant,
/// discriminant 36). PROTOCOL_VERSION ALSO moved 21 -> 22 -- a deviation from the
/// plan's prediction of no PROTOCOL bump: `EffectFilter`'s sibling field
/// `EffectDuration` (same `ContinuousEffectDef` struct) already put the struct in the
/// wire closure at PB-EF9 (v14), so `EffectFilter`'s new variant is reachable too. See
/// `crates/engine/src/rules/protocol.rs`'s `- 22:` history line for the full account.
#[test]
fn test_os7_version_sentinels() {
    assert_eq!(
        HASH_SCHEMA_VERSION, 59u8,
        "HASH_SCHEMA_VERSION should be 59 after PB-OS7 (EffectFilter gained \
         CreaturesControlledByDefendingPlayer, discriminant 36)"
    );
    assert_eq!(
        PROTOCOL_VERSION, 22,
        "PROTOCOL_VERSION should be 22 after PB-OS7 -- see the protocol.rs `- 22:` \
         history line for why this moved despite the plan predicting no bump"
    );
}
