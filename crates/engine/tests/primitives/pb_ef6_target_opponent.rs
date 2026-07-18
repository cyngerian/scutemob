//! PB-EF6 (scutemob-107): `TargetRequirement::TargetOpponent` (EF-W-PB2-2).
//!
//! CR 102.2/102.3 (opponent definition — no teams model, so opponent = any player
//! other than the controller), CR 115.1 / 601.2c (targeting / declaration-time
//! restriction), CR 603.3d (trigger removed from the stack if no legal target
//! exists — the auto-target picker must NOT fall back to the controller).
//!
//! HASH_SCHEMA_VERSION 48 -> 49, PROTOCOL_VERSION 10 -> 11 (new `TargetRequirement`
//! unit variant reaches both the GameState hash closure and the SR-8 protocol
//! fingerprint closure).
//!
//! Card integration: shaman_of_the_pack, raiders_wake, vengeful_bloodwitch (all
//! flipped `inert`/`partial`/`known_wrong` -> Complete by this batch) and
//! fell_specter (stayed Complete; a latent self-targetable bug on its ETB is fixed).

use std::collections::HashMap;
use std::sync::Arc;

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stubs::{PendingTrigger, PendingTriggerKind};
use mtg_engine::{
    enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition, CardEffectTarget,
    CardId, CardRegistry, CardType, Command, Effect, GameEvent, GameState, GameStateBuilder,
    GameStateError, ManaCost, ManaPool, ObjectId, ObjectSpec, PlayerId, SpellTarget, Step, SubType,
    Target, TargetRequirement, TypeLine, ZoneId, HASH_SCHEMA_VERSION,
};

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::abilities::{check_triggers, flush_pending_triggers};

// ── Helpers ──────────────────────────────────────────────────────────────────

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

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn life(state: &GameState, player: PlayerId) -> i32 {
    state
        .players()
        .get(&player)
        .map(|p| p.life_total)
        .unwrap_or_default()
}

fn defs_of(def: &CardDefinition) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
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

/// Pass priority repeatedly until `target` step is reached.
fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn().step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (infinite loop?)"
        );
        let holder = state.turn().priority_holder.expect("no priority holder");
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
    }
}

/// Resolve everything currently on the stack.
fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects().is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

fn cast_spell(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> Result<(GameState, Vec<GameEvent>), GameStateError> {
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
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
}

/// A minimal instant "target opponent" test spell — the effect does nothing;
/// only the target validation path is under test.
fn opponent_target_test_spell() -> CardDefinition {
    CardDefinition {
        name: "EF6 Target Opponent Test Spell".to_string(),
        card_id: CardId("test-ef6-target-opponent-spell".to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Nothing,
            targets: vec![TargetRequirement::TargetOpponent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a fresh 4-player state with `opponent_target_test_spell` in p1's hand.
/// Call once per cast attempt so each scenario gets its own uncast copy of the spell.
fn build_opponent_spell_base_state() -> (GameState, ObjectId) {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let spell_def = opponent_target_test_spell();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let spell = ObjectSpec::card(p1, "EF6 Target Opponent Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone());

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 1,
                ..ManaPool::default()
            },
        )
        .object(spell)
        .build()
        .expect("build_opponent_spell_base_state: GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "EF6 Target Opponent Test Spell");
    (state, spell_id)
}

// ── Test 1: the required 4-player accept-opponent / reject-self test ──────────

/// CR 601.2c — a spell with `targets: vec![TargetRequirement::TargetOpponent]`:
/// - self-target (P1 targeting P1) is rejected with `GameStateError::InvalidTarget`.
/// - any of the three other players (P2, P3, P4) is a legal target and the cast
///   succeeds (the spell moves onto the stack).
///
/// Exercises the Step 3 `validate_player_satisfies_requirement` path directly —
/// this is the declaration-time restriction, independent of the auto-target picker.
#[test]
fn test_target_opponent_spell_accepts_opponent_rejects_self_4p() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    // Self-target: rejected.
    {
        let (state, spell_id) = build_opponent_spell_base_state();
        let result = cast_spell(state, p1, spell_id, vec![Target::Player(p1)]);
        match result {
            Err(GameStateError::InvalidTarget(_)) => {}
            other => panic!(
                "CR 601.2c/102.3: self-targeting TargetOpponent must be rejected as \
                 InvalidTarget, got: {:?}",
                other.map(|_| ())
            ),
        }
    }

    // Each of the three other players is a legal target.
    for &opponent in &[p2, p3, p4] {
        let (state, spell_id) = build_opponent_spell_base_state();
        let (state, _) = cast_spell(state, p1, spell_id, vec![Target::Player(opponent)])
            .unwrap_or_else(|e| {
                panic!(
                    "CR 601.2c: casting TargetOpponent at opponent {:?} must succeed: {:?}",
                    opponent, e
                )
            });
        assert!(
            !state.stack_objects().is_empty(),
            "spell targeting opponent {:?} must be on the stack after a successful cast",
            opponent
        );
    }
}

// ── Test 2: decoy — fails on EXACTLY the opponent-ness field ──────────────────

/// Decoy test (SR-36 sense): this test PASSES if and only if `TargetOpponent`
/// restricts to non-caster players. If the validation arm were accidentally
/// aliased to `TargetPlayer` semantics (`Ok(())` for everyone, including the
/// caster), this test would go RED against the pre-fix engine only because we
/// assert the self-target is rejected — do not weaken this assertion to
/// "cast succeeds" or the decoy is defeated.
#[test]
fn test_target_opponent_decoy_self_must_be_rejected() {
    let p1 = p(1);
    let (state, spell_id) = build_opponent_spell_base_state();
    let result = cast_spell(state, p1, spell_id, vec![Target::Player(p1)]);
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "DECOY: self-target for TargetOpponent must be Err(InvalidTarget); a validation \
         arm that treats TargetOpponent as unrestricted (like TargetPlayer) would \
         wrongly return Ok here."
    );
}

// ── Test 3: hash distinctness + live schema sentinel ───────────────────────────

/// HASH_SCHEMA_VERSION live sentinel (48 -> 49) — see `state/hash.rs` history
/// block. `TargetRequirement::TargetOpponent` (discriminant 18) must hash
/// distinctly from `TargetPlayer` (discriminant 1) and `UpToN` (discriminant 17).
#[test]
fn test_target_opponent_hashes_distinctly() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    assert_eq!(
        HASH_SCHEMA_VERSION, 53u8,
        "HASH_SCHEMA_VERSION drifted without this sentinel being updated. Bump this \
         assertion and the state/hash.rs history block together; the authoritative \
         check is the SR-17 machine gate in tests/core/hash_schema.rs."
    );

    let hash_req = |req: &TargetRequirement| -> [u8; 32] {
        let mut hasher = Hasher::new();
        req.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    let opponent = TargetRequirement::TargetOpponent;
    let player = TargetRequirement::TargetPlayer;
    let upto_n_creature = TargetRequirement::UpToN {
        count: 1,
        inner: Box::new(TargetRequirement::TargetCreature),
    };

    let h_opponent = hash_req(&opponent);
    let h_player = hash_req(&player);
    let h_upto_n = hash_req(&upto_n_creature);

    assert_ne!(
        h_opponent, h_player,
        "TargetOpponent (disc 18) must hash distinctly from TargetPlayer (disc 1)"
    );
    assert_ne!(
        h_opponent, h_upto_n,
        "TargetOpponent (disc 18) must hash distinctly from UpToN (disc 17)"
    );
}

// ── Test 4: Shaman of the Pack — ETB targets an opponent, correct life loss ────

/// CR 603.3d + `EffectAmount::PermanentCount` — "When this creature enters, target
/// opponent loses life equal to the number of Elves you control." Shaman of the
/// Pack is itself an Elf Shaman, so once it has resolved onto the battlefield the
/// count includes itself: 0 other Elves -> count 1; 2 other Elves -> count 3.
/// Both cases assert the loser is an OPPONENT (never P1, the caster).
#[test]
fn test_shaman_of_the_pack_etb_targets_opponent_loses_life() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::shaman_of_the_pack::card();
    let defs = defs_of(&def);

    // -- 0 other Elves: Shaman counts itself -> exactly 1 life lost. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(enrich_spec_from_def(
                ObjectSpec::card(p1, "Shaman of the Pack")
                    .with_card_id(def.card_id.clone())
                    .in_zone(ZoneId::Hand(p1)),
                &defs,
            ))
            .player_mana(
                p1,
                ManaPool {
                    colorless: 1,
                    black: 1,
                    green: 1,
                    ..ManaPool::default()
                },
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .expect("0-elf: GameStateBuilder::build must succeed");

        let spell_id = find_obj(&state, "Shaman of the Pack");
        let p1_life_before = life(&state, p1);
        let p2_life_before = life(&state, p2);

        let (state, _) =
            cast_spell(state, p1, spell_id, vec![]).expect("casting Shaman must succeed");
        let (state, _) = pass_all(state, &[p1, p2]); // resolve creature spell -> ETB queued
        let (state, _) = pass_all(state, &[p1, p2]); // resolve ETB trigger

        assert_eq!(
            life(&state, p2),
            p2_life_before - 1,
            "0 other Elves: opponent must lose exactly 1 life (Shaman counts itself)"
        );
        assert_eq!(
            life(&state, p1),
            p1_life_before,
            "the auto-picked target must be an OPPONENT, never the caster P1"
        );
    }

    // -- 2 other Elves (P1-controlled) + 1 Elf on the OPPONENT's side: total
    // Elves-you-control = 3 (2 others + itself); P2's own Elf must NOT count
    // (proves `controller: You`, not `Any` -- LOW-2 / review Finding 2). --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(enrich_spec_from_def(
                ObjectSpec::card(p1, "Shaman of the Pack")
                    .with_card_id(def.card_id.clone())
                    .in_zone(ZoneId::Hand(p1)),
                &defs,
            ))
            .object(
                ObjectSpec::creature(p1, "Other Elf A", 1, 1)
                    .with_subtypes(vec![SubType("Elf".to_string())]),
            )
            .object(
                ObjectSpec::creature(p1, "Other Elf B", 1, 1)
                    .with_subtypes(vec![SubType("Elf".to_string())]),
            )
            .object(
                ObjectSpec::creature(p2, "Opponent's Elf", 1, 1)
                    .with_subtypes(vec![SubType("Elf".to_string())]),
            )
            .player_mana(
                p1,
                ManaPool {
                    colorless: 1,
                    black: 1,
                    green: 1,
                    ..ManaPool::default()
                },
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .expect("2-elf: GameStateBuilder::build must succeed");

        let spell_id = find_obj(&state, "Shaman of the Pack");
        let p2_life_before = life(&state, p2);

        let (state, _) =
            cast_spell(state, p1, spell_id, vec![]).expect("casting Shaman must succeed");
        let (state, _) = pass_all(state, &[p1, p2]);
        let (state, _) = pass_all(state, &[p1, p2]);

        assert_eq!(
            life(&state, p2),
            p2_life_before - 3,
            "2 P1-controlled other Elves + Shaman itself = 3: opponent must lose exactly 3 \
             life -- the opponent's OWN Elf must be excluded (controller: You, not Any)"
        );
    }
}

// ── Test 5: Raiders' Wake — Raid targets an opponent, gated by attacked_this_turn ─

/// CR 508.1 (Raid) + PB-EF6 — "At the beginning of your end step, if you attacked
/// this turn, target opponent discards a card." Verifies the discard fires
/// (against an opponent, not P1) when `attacked_this_turn` is true, and that the
/// intervening-if suppresses the trigger entirely (no discard) when false.
#[test]
fn test_raiders_wake_raid_targets_opponent_discards() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::raiders_wake::card();
    let defs = defs_of(&def);

    // -- attacked_this_turn = true: opponent discards exactly 1. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(enrich_spec_from_def(
                ObjectSpec::card(p1, "Raiders' Wake")
                    .with_card_id(def.card_id.clone())
                    .in_zone(ZoneId::Battlefield),
                &defs,
            ))
            .object(ObjectSpec::card(p2, "Filler Card").in_zone(ZoneId::Hand(p2)))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .expect("attacked=true: GameStateBuilder::build must succeed");
        state.players_mut().get_mut(&p1).unwrap().attacked_this_turn = true;

        let state = advance_to_step(state, Step::End);
        let state = resolve_stack(state, &[p1, p2]);

        assert!(
            find_in_zone(&state, "Filler Card", ZoneId::Graveyard(p2)).is_some(),
            "Raid: attacked_this_turn=true must discard the OPPONENT's (P2's) card"
        );
        assert!(
            find_in_zone(&state, "Filler Card", ZoneId::Hand(p2)).is_none(),
            "the discarded card must have left P2's hand"
        );
    }

    // -- attacked_this_turn = false (default): intervening-if suppresses the trigger. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(enrich_spec_from_def(
                ObjectSpec::card(p1, "Raiders' Wake")
                    .with_card_id(def.card_id.clone())
                    .in_zone(ZoneId::Battlefield),
                &defs,
            ))
            .object(ObjectSpec::card(p2, "Filler Card").in_zone(ZoneId::Hand(p2)))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .expect("attacked=false: GameStateBuilder::build must succeed");

        let state = advance_to_step(state, Step::End);
        let state = resolve_stack(state, &[p1, p2]);

        assert!(
            find_in_zone(&state, "Filler Card", ZoneId::Hand(p2)).is_some(),
            "Raid: attacked_this_turn=false must NOT discard (intervening-if gates it)"
        );
    }
}

// ── Test 6: Vengeful Bloodwitch — death trigger targets an opponent ───────────

/// CR 603.10a + PB-EF6 — "Whenever this creature or another creature you control
/// dies, target opponent loses 1 life and you gain 1 life." Proves the
/// known_wrong -> Complete flip is correct, not vacuous: the death of a second
/// creature P1 controls must make an OPPONENT lose life, never P1.
#[test]
fn test_vengeful_bloodwitch_death_trigger_targets_opponent() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::vengeful_bloodwitch::card();
    let defs = defs_of(&def);
    let registry = CardRegistry::new(vec![def.clone()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Vengeful Bloodwitch")
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs,
        ))
        .object(ObjectSpec::creature(p1, "Fodder Creature", 1, 1))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let fodder_id = find_obj(&state, "Fodder Creature");
    let p1_life_before = life(&state, p1);
    let p2_life_before = life(&state, p2);

    let mut ctx = EffectContext::new(
        p1,
        fodder_id,
        vec![SpellTarget {
            target: Target::Object(fodder_id),
            zone_at_cast: Some(ZoneId::Battlefield),
        }],
    );
    let destroy_events = execute_effect(
        &mut state,
        &Effect::DestroyPermanent {
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            cant_be_regenerated: false,
        },
        &mut ctx,
    );
    assert!(
        destroy_events
            .iter()
            .any(|e| matches!(e, GameEvent::CreatureDied { .. })),
        "sanity: Fodder Creature's death must emit CreatureDied"
    );

    let triggers = check_triggers(&state, &destroy_events);
    assert!(
        !triggers.is_empty(),
        "sanity: Vengeful Bloodwitch's death trigger must be queued"
    );
    for t in triggers {
        state.pending_triggers_mut().push_back(t);
    }
    let flush_events = flush_pending_triggers(&mut state);
    assert!(
        flush_events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "Vengeful Bloodwitch's death trigger must be placed on the stack"
    );

    // Resolve the stack (LoseLife + GainLife sequence).
    let state = resolve_stack(state, &[p1, p2]);

    assert_eq!(
        life(&state, p2),
        p2_life_before - 1,
        "the auto-picked OPPONENT (P2) must lose 1 life"
    );
    assert_eq!(
        life(&state, p1),
        p1_life_before + 1,
        "the controller (P1) must gain 1 life"
    );
}

// ── Test 7: Fell Specter — ETB regression, no longer self-targetable ──────────

/// Regression pin for the latent bug fixed by this batch: Fell Specter's ETB
/// ("target opponent discards a card") previously used `TargetRequirement::
/// TargetPlayer`, which the auto-picker resolved with a self-fallback — legal
/// but wrong (CR 601.2c) if it ever had no opponent, and generally an
/// unenforced restriction. This test pins the corrected auto-picker: the
/// discarding player must be P2 (the opponent), never P1 (the caster).
#[test]
fn test_fell_specter_etb_no_longer_self_targetable() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::fell_specter::card();
    let defs = defs_of(&def);
    let registry = CardRegistry::new(vec![def.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Fell Specter")
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        ))
        .object(ObjectSpec::card(p2, "Filler Card").in_zone(ZoneId::Hand(p2)))
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                black: 1,
                ..ManaPool::default()
            },
        )
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "Fell Specter");
    let (state, _) =
        cast_spell(state, p1, spell_id, vec![]).expect("casting Fell Specter must succeed");
    let (state, _) = pass_all(state, &[p1, p2]); // resolve creature spell -> ETB queued
    let (state, _) = pass_all(state, &[p1, p2]); // resolve ETB discard trigger

    assert!(
        find_in_zone(&state, "Filler Card", ZoneId::Graveyard(p2)).is_some(),
        "regression: Fell Specter's ETB must target an OPPONENT (P2), never self-target P1"
    );
    assert!(
        find_in_zone(&state, "Filler Card", ZoneId::Hand(p2)).is_none(),
        "the discarded card must have left P2's hand"
    );
}

// ── Test 8: no legal opponent -- trigger removed from the stack, NO self-fallback ─

/// CR 603.3d — a contrived state where the only possible opponent has left the
/// game (`has_lost = true`): a `TargetOpponent` trigger has NO legal candidate.
/// The trigger must be removed (never placed on the stack), and critically must
/// NOT fall back to targeting the controller. Decoy for the auto-picker's
/// no-self-fallback guarantee (DECISION 3): a picker that fell back to
/// `trigger.controller` would push a stack item and, upon resolution, P1 would
/// lose life to their own trigger — this test would go red on that behavior.
#[test]
fn test_target_opponent_trigger_no_opponent_removed_from_stack() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::shaman_of_the_pack::card();
    let defs = defs_of(&def);
    let registry = CardRegistry::new(vec![def.clone()]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Shaman of the Pack")
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Battlefield),
            &defs,
        ))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .expect("GameStateBuilder::build must succeed");

    // The only possible opponent has left the game (CR 800.4a) -- no legal
    // TargetOpponent candidate exists anywhere in state.
    state.players_mut().get_mut(&p2).unwrap().has_lost = true;

    let shaman_id = find_obj(&state, "Shaman of the Pack");
    let p1_life_before = life(&state, p1);

    // Manually inject the ETB PendingTrigger. `PendingTriggerKind::CardDefETB`
    // indexes `def.abilities` by raw index -- ability_index 0 is Shaman's only
    // ability. This exercises `flush_pending_triggers`'s auto-target picker in
    // isolation (SR-7: `PendingTrigger::blank` is the only supported way to
    // construct one in tests).
    let trigger = PendingTrigger {
        ability_index: 0,
        ..PendingTrigger::blank(shaman_id, p1, PendingTriggerKind::CardDefETB)
    };
    state.pending_triggers_mut().push_back(trigger);

    let events = flush_pending_triggers(&mut state);

    // CR 603.3d: no legal target -> the ability is removed, never placed on the
    // stack. NEVER fall back to trigger.controller (P1).
    assert!(
        state.stack_objects().is_empty(),
        "CR 603.3d: with no legal opponent, the trigger must be removed, not pushed \
         to the stack (NO self-fallback)"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, GameEvent::AbilityTriggered { .. })),
        "CR 603.3d: no AbilityTriggered event should be emitted when no legal target exists"
    );
    assert!(
        state.pending_triggers().is_empty(),
        "the pending trigger must be drained (processed), even though it produced no stack item"
    );
    // P1's life must be untouched -- proves no self-fallback resolved against P1.
    assert_eq!(
        life(&state, p1),
        p1_life_before,
        "P1 must not lose life -- the trigger must not have targeted the controller"
    );
}

// ── Test 9: object target rejected -- a player requirement can't take an object ─

/// CR 601.2c — a player-targeting requirement cannot be satisfied by an object
/// target. `validate_object_satisfies_requirement`'s exhaustive `valid` match
/// (casting.rs) has `TargetPlayer | TargetOpponent => false`; this test exercises
/// that arm at runtime rather than relying solely on compile-time exhaustiveness.
#[test]
fn test_target_opponent_rejects_object_target() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let spell_def = opponent_target_test_spell();
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let spell = ObjectSpec::card(p1, "EF6 Target Opponent Test Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone());
    let creature =
        ObjectSpec::creature(p2, "Bystander Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 1,
                ..ManaPool::default()
            },
        )
        .object(spell)
        .object(creature)
        .build()
        .expect("GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "EF6 Target Opponent Test Spell");
    let creature_id = find_obj(&state, "Bystander Creature");

    let result = cast_spell(state, p1, spell_id, vec![Target::Object(creature_id)]);
    assert!(
        matches!(result, Err(GameStateError::InvalidTarget(_))),
        "CR 601.2c: an object target must be rejected for TargetOpponent (a player \
         requirement), got: {:?}",
        result.map(|_| ())
    );
}
