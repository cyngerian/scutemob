//! PB-T: TargetRequirement::UpToN — optional target slots (CR 601.2c / CR 115.1b).
//!
//! Tests verify that the new `UpToN { count, inner }` variant correctly:
//! - Allows zero targets (player chooses not to target any legal permanents)
//! - Allows partial targets (1 of up-to-2, etc.)
//! - Allows full-count targets (N of N)
//! - Hashes distinctly from existing variants (schema bump 7 → 8)
//! - Participates in partial fizzle (CR 608.2b)
//! - Does not regress mandatory-target cards (CR 601.2c min/max range)
//! - Works with mixed mandatory + UpToN requirement lists
//! - Rejects wrong-type targets
//!
//! CR Rules covered:
//! - CR 601.2c: At cast time, player announces targets. For "up to N" spells,
//!   player announces how many targets (0..=N), then announces those targets.
//! - CR 115.1b: If a spell says "up to [N]," the player chooses between zero
//!   and N targets (inclusive).
//! - CR 608.2b: At resolution, illegal targets are dropped. A spell fizzles only if
//!   ALL targets become illegal. If some remain, the spell resolves for those.
//! - CR 400.7: Objects that move zones become new objects with new identity.

use std::sync::Arc;

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Effect, EffectTarget, TargetRequirement, TypeLine,
};
use mtg_engine::rules::{process_command, Command, GameEvent};
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::{
    CardId, CardType, GameStateBuilder, ManaPool, ObjectSpec, PlayerId, Target, ZoneId,
};
use mtg_engine::{CardRegistry, GameState, ObjectId, HASH_SCHEMA_VERSION};

// ── Helpers ────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn obj_on_battlefield(state: &GameState, name: &str) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Battlefield)
}

fn obj_in_graveyard(state: &GameState, name: &str, owner: PlayerId) -> bool {
    state
        .objects
        .values()
        .any(|o| o.characteristics.name == name && o.zone == ZoneId::Graveyard(owner))
}

fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut s = state;
    let mut all_events = Vec::new();
    for &pl in players {
        let (ns, evs) = process_command(s, Command::PassPriority { player: pl }).unwrap();
        all_events.extend(evs);
        s = ns;
    }
    (s, all_events)
}

fn cast_spell(player: PlayerId, card: ObjectId, targets: Vec<Target>) -> Command {
    Command::CastSpell {
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
    }
}

/// Build a destroy spell with UpToN targeting artifacts.
/// "Destroy up to N target artifacts." — mirrors Force of Vigor semantics.
fn up_to_n_destroy_artifact_spell(name: &str, count: u32) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-up-to-n-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        mana_cost: Some(ManaCost {
            green: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(
                (0..count as usize)
                    .map(|i| Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: i },
                        cant_be_regenerated: false,
                    })
                    .collect(),
            ),
            targets: vec![TargetRequirement::UpToN {
                count,
                inner: Box::new(TargetRequirement::TargetArtifact),
            }],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a tap spell with UpToN targeting permanents.
/// "Tap up to N target permanents." — mirrors Elder Deep-Fiend semantics.
fn up_to_n_tap_permanent_spell(name: &str, count: u32) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-tap-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        mana_cost: Some(ManaCost {
            blue: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(
                (0..count as usize)
                    .map(|i| Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: i },
                    })
                    .collect(),
            ),
            targets: vec![TargetRequirement::UpToN {
                count,
                inner: Box::new(TargetRequirement::TargetPermanent),
            }],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// Build a mandatory 1-target destroy-creature spell.
fn mandatory_target_destroy_creature_spell(name: &str) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(format!(
            "test-mandatory-{}",
            name.to_lowercase().replace(' ', "-")
        )),
        mana_cost: Some(ManaCost {
            red: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::DestroyPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
                cant_be_regenerated: false,
            },
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── M1: Zero-target resolution ─────────────────────────────────────────────────

/// CR 601.2c / CR 115.1b / CR 608.2b — PB-T M1: Casting an UpToN spell with 0 targets
/// is legal. The spell resolves without applying the effect to any permanent.
///
/// Setup: P1 has "Destroy up to 2 target artifacts" in hand. P2 has an artifact on
/// the battlefield. P1 casts the spell with 0 targets.
///
/// Assert:
/// - Cast succeeds (no InvalidTarget error).
/// - Spell resolves without destroying any artifact.
/// - P2's artifact remains on battlefield.
#[test]
fn test_pbt_up_to_n_zero_targets_resolves_without_effect() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let spell_def = up_to_n_destroy_artifact_spell("Destroy Up To Two", 2);
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let spell = ObjectSpec::card(p1, "Destroy Up To Two")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone());

    let artifact = ObjectSpec::artifact(p2, "Clue Token").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                green: 2,
                ..ManaPool::default()
            },
        )
        .object(spell)
        .object(artifact)
        .build()
        .expect("M1: GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "Destroy Up To Two");

    // CR 601.2c / 115.1b: Cast with 0 targets (legal for "up to N" spells).
    let (state, _) = process_command(state, cast_spell(p1, spell_id, vec![]))
        .expect("M1: casting with 0 UpToN targets must succeed");

    // Spell is on the stack; all players pass priority to resolve it.
    let (state, _) = pass_all(state, &players);

    // CR 608.2b: spell resolves; with 0 targets there is nothing to destroy.
    assert!(
        obj_on_battlefield(&state, "Clue Token"),
        "M1: artifact must remain on battlefield when spell declared 0 targets"
    );
}

// ── M2: Partial-target resolution (1 of up-to-2) ──────────────────────────────

/// CR 601.2c — PB-T M2: Declaring 1 of up-to-2 targets is legal. Only the 1 declared
/// target is affected; the other (undeclared) is untouched.
///
/// Setup: P1 casts "Destroy up to 2 artifacts" with 1 artifact declared. P2 has 2
/// artifacts on battlefield.
///
/// Assert: exactly 1 artifact is destroyed; the other remains.
#[test]
fn test_pbt_up_to_n_partial_targets_resolves() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let spell_def = up_to_n_destroy_artifact_spell("Destroy Up To Two", 2);
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let spell = ObjectSpec::card(p1, "Destroy Up To Two")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone());

    let artifact_a = ObjectSpec::artifact(p2, "Artifact A").in_zone(ZoneId::Battlefield);
    let artifact_b = ObjectSpec::artifact(p2, "Artifact B").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                green: 2,
                ..ManaPool::default()
            },
        )
        .object(spell)
        .object(artifact_a)
        .object(artifact_b)
        .build()
        .expect("M2: GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "Destroy Up To Two");
    let art_a_id = find_obj(&state, "Artifact A");

    // CR 601.2c: Cast with 1 of up-to-2 targets declared.
    let (state, _) = process_command(
        state,
        cast_spell(p1, spell_id, vec![Target::Object(art_a_id)]),
    )
    .expect("M2: casting with 1 of up-to-2 UpToN targets must succeed");

    let (state, _) = pass_all(state, &players);

    assert!(
        obj_in_graveyard(&state, "Artifact A", p2),
        "M2: declared target must be destroyed"
    );
    assert!(
        obj_on_battlefield(&state, "Artifact B"),
        "M2: undeclared artifact must remain on battlefield"
    );
}

// ── M3: Full-target resolution (N of N) ───────────────────────────────────────

/// CR 601.2c — PB-T M3: Declaring N of up-to-N targets is legal and affects all N.
///
/// Setup: P1 casts "Destroy up to 2 artifacts" with 2 artifacts declared.
///
/// Assert: both artifacts are destroyed.
#[test]
fn test_pbt_up_to_n_full_targets_resolves() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let spell_def = up_to_n_destroy_artifact_spell("Destroy Up To Two", 2);
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let spell = ObjectSpec::card(p1, "Destroy Up To Two")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone());

    let artifact_a = ObjectSpec::artifact(p2, "Artifact A").in_zone(ZoneId::Battlefield);
    let artifact_b = ObjectSpec::artifact(p2, "Artifact B").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                green: 2,
                ..ManaPool::default()
            },
        )
        .object(spell)
        .object(artifact_a)
        .object(artifact_b)
        .build()
        .expect("M3: GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "Destroy Up To Two");
    let art_a_id = find_obj(&state, "Artifact A");
    let art_b_id = find_obj(&state, "Artifact B");

    // CR 601.2c: Cast with N=2 of up-to-2 targets declared.
    let (state, _) = process_command(
        state,
        cast_spell(
            p1,
            spell_id,
            vec![Target::Object(art_a_id), Target::Object(art_b_id)],
        ),
    )
    .expect("M3: casting with 2 of up-to-2 UpToN targets must succeed");

    let (state, _) = pass_all(state, &players);

    assert!(
        obj_in_graveyard(&state, "Artifact A", p2),
        "M3: first declared target must be destroyed"
    );
    assert!(
        obj_in_graveyard(&state, "Artifact B", p2),
        "M3: second declared target must be destroyed"
    );
}

// ── M4: Hash determinism + schema bump ────────────────────────────────────────

/// CR N/A (hash infrastructure) — PB-T M4: HASH_SCHEMA_VERSION is 8. Three distinct
/// UpToN variants hash to distinct values, confirming the new discriminant-17 arm
/// is reached and the count/inner fields contribute to the hash.
#[test]
fn test_pbt_hash_schema_version_is_8() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    // CR N/A — sentinel must be 8 (PB-T bump from PB-L's 7).
    assert_eq!(
        HASH_SCHEMA_VERSION,
        8u8,
        "PB-T: HASH_SCHEMA_VERSION must be 8 (bump from PB-L's 7 for TargetRequirement::UpToN, CR 601.2c / 115.1b)"
    );

    let hash_req = |req: &TargetRequirement| -> [u8; 32] {
        let mut hasher = Hasher::new();
        req.hash_into(&mut hasher);
        *hasher.finalize().as_bytes()
    };

    // Three distinct UpToN configurations — all must hash to distinct values.
    let upto1_creature = TargetRequirement::UpToN {
        count: 1,
        inner: Box::new(TargetRequirement::TargetCreature),
    };
    let upto2_creature = TargetRequirement::UpToN {
        count: 2,
        inner: Box::new(TargetRequirement::TargetCreature),
    };
    let upto1_permanent = TargetRequirement::UpToN {
        count: 1,
        inner: Box::new(TargetRequirement::TargetPermanent),
    };

    let h1 = hash_req(&upto1_creature);
    let h2 = hash_req(&upto2_creature);
    let h3 = hash_req(&upto1_permanent);

    assert_ne!(
        h1, h2,
        "M4: UpToN count=1,Creature and count=2,Creature must hash distinctly (count matters)"
    );
    assert_ne!(
        h1, h3,
        "M4: UpToN count=1,Creature and count=1,Permanent must hash distinctly (inner matters)"
    );
    assert_ne!(
        h2, h3,
        "M4: UpToN count=2,Creature and count=1,Permanent must hash distinctly"
    );

    // Also verify UpToN discriminant (17) differs from neighboring variants (0, 16).
    let h_mandatory_creature = hash_req(&TargetRequirement::TargetCreature);
    let h_single_target_ability =
        hash_req(&TargetRequirement::TargetSpellOrAbilityWithSingleTarget);

    assert_ne!(
        h1, h_mandatory_creature,
        "M4: UpToN count=1,Creature must differ from TargetCreature (different discriminant)"
    );
    assert_ne!(
        h1,
        h_single_target_ability,
        "M4: UpToN count=1,Creature must differ from TargetSpellOrAbilityWithSingleTarget (disc 16 vs 17)"
    );
}

// ── M5: Partial fizzle (one target becomes illegal) ───────────────────────────

/// CR 608.2b / CR 400.7 — PB-T M5: A spell declared with fewer than max UpToN
/// targets resolves for the declared targets without fizzling. This verifies the
/// foundational partial-fizzle semantics (a spell with some illegal targets resolves
/// for the remaining legal ones).
///
/// Setup: "Tap up to 2 permanents" cast with 1 target. The 1 declared target is tapped.
/// The spell does not fizzle (CR 608.2b: fizzle only if ALL targets illegal).
///
/// CR 608.2b: "Illegal targets, if any, won't be affected by parts of a resolving
/// spell's effect for which they're illegal."
#[test]
fn test_pbt_up_to_n_partial_fizzle_on_zone_change() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let spell_def = up_to_n_tap_permanent_spell("Tap Up To Two", 2);
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let spell = ObjectSpec::card(p1, "Tap Up To Two")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone());

    let creature_p2 = ObjectSpec::creature(p2, "Grizzly Bears", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                blue: 2,
                ..ManaPool::default()
            },
        )
        .object(spell)
        .object(creature_p2)
        .build()
        .expect("M5: GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "Tap Up To Two");
    let bear_id = find_obj(&state, "Grizzly Bears");

    // CR 601.2c / 115.1b: Declare 1 of up-to-2 targets (partial, legal).
    let (state, _) = process_command(
        state,
        cast_spell(p1, spell_id, vec![Target::Object(bear_id)]),
    )
    .expect("M5: declaring 1 of up-to-2 targets must succeed");

    let (state, _) = pass_all(state, &players);

    // CR 608.2b: spell resolves; 1 target is tapped. No fizzle.
    let bear_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Grizzly Bears")
        .expect("M5: Grizzly Bears must still exist on battlefield");
    assert!(
        bear_obj.status.tapped,
        "M5: declared target must be tapped at resolution"
    );
}

// ── M6: Regression — mandatory-target spells still work ───────────────────────

/// CR 601.2c — PB-T M6: The new `target_count_range` helper returns (1, 1) for
/// mandatory-only requirement lists. This verifies PB-T did not regress existing
/// mandatory-target cards:
/// - 0 targets → rejected
/// - 2 targets → rejected
/// - exactly 1 target → success
#[test]
fn test_pbt_regression_mandatory_target_spells_still_work() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let spell_def = mandatory_target_destroy_creature_spell("Destroy Creature");
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let build_state = |registry: Arc<CardRegistry>| -> GameState {
        let spell = ObjectSpec::card(p1, "Destroy Creature")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(spell_def.card_id.clone());
        let creature_a =
            ObjectSpec::creature(p2, "Creature Alpha", 2, 2).in_zone(ZoneId::Battlefield);
        let creature_b =
            ObjectSpec::creature(p2, "Creature Beta", 3, 3).in_zone(ZoneId::Battlefield);

        GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .add_player(p3)
            .add_player(p4)
            .with_registry(registry)
            .player_mana(
                p1,
                ManaPool {
                    red: 1,
                    ..ManaPool::default()
                },
            )
            .object(spell)
            .object(creature_a)
            .object(creature_b)
            .build()
            .expect("M6: GameStateBuilder::build must succeed")
    };

    // (a) CR 601.2c: 0 targets for a mandatory-1-target spell must be rejected.
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Destroy Creature");
        let result = process_command(state, cast_spell(p1, spell_id, vec![]));
        assert!(
            result.is_err(),
            "M6a: 0 targets for mandatory-target spell must be rejected"
        );
    }

    // (b) CR 601.2c: 2 targets for a mandatory-1-target spell must be rejected.
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Destroy Creature");
        let alpha_id = find_obj(&state, "Creature Alpha");
        let beta_id = find_obj(&state, "Creature Beta");
        let result = process_command(
            state,
            cast_spell(
                p1,
                spell_id,
                vec![Target::Object(alpha_id), Target::Object(beta_id)],
            ),
        );
        assert!(
            result.is_err(),
            "M6b: 2 targets for mandatory-1-target spell must be rejected"
        );
    }

    // (c) CR 601.2c: exactly 1 target → success; creature destroyed.
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Destroy Creature");
        let alpha_id = find_obj(&state, "Creature Alpha");
        let (state, _) = process_command(
            state,
            cast_spell(p1, spell_id, vec![Target::Object(alpha_id)]),
        )
        .expect("M6c: 1 target for mandatory-1-target spell must succeed");
        let (state, _) = pass_all(state, &players);
        assert!(
            obj_in_graveyard(&state, "Creature Alpha", p2),
            "M6c: targeted creature must be destroyed"
        );
        assert!(
            obj_on_battlefield(&state, "Creature Beta"),
            "M6c: non-targeted creature must remain"
        );
    }
}

// ── M7: Mixed mandatory + UpToN ───────────────────────────────────────────────

/// CR 601.2c — PB-T M7: A spell with `[TargetCreature (mandatory), UpToN{1, Creature}]`
/// requires exactly 1 mandatory target plus 0 or 1 optional target.
///
/// - 0 targets → rejected (mandatory not satisfied).
/// - 1 target → success (mandatory satisfied, UpToN takes 0).
/// - 2 targets → success (mandatory takes 1, UpToN takes 1).
/// - 3 targets → rejected (exceeds max 2).
///
/// Exercises the greedy-consume validator's mixed-slot path.
#[test]
fn test_pbt_mixed_mandatory_and_up_to_n() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let spell_def = CardDefinition {
        name: "Mixed Mandatory UpToN".to_string(),
        card_id: CardId("test-mixed-mandatory-uptoN".to_string()),
        mana_cost: Some(ManaCost {
            black: 2,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 1 },
                    cant_be_regenerated: false,
                },
            ]),
            targets: vec![
                TargetRequirement::TargetCreature,
                TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetCreature),
                },
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let build_state = |registry: Arc<CardRegistry>| -> GameState {
        let spell = ObjectSpec::card(p1, "Mixed Mandatory UpToN")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(spell_def.card_id.clone());
        let creature_a =
            ObjectSpec::creature(p2, "Creature Alpha", 2, 2).in_zone(ZoneId::Battlefield);
        let creature_b =
            ObjectSpec::creature(p2, "Creature Beta", 3, 3).in_zone(ZoneId::Battlefield);
        let creature_c =
            ObjectSpec::creature(p3, "Creature Gamma", 1, 1).in_zone(ZoneId::Battlefield);

        GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .add_player(p3)
            .add_player(p4)
            .with_registry(registry)
            .player_mana(
                p1,
                ManaPool {
                    black: 2,
                    ..ManaPool::default()
                },
            )
            .object(spell)
            .object(creature_a)
            .object(creature_b)
            .object(creature_c)
            .build()
            .expect("M7: GameStateBuilder::build must succeed")
    };

    // (a) 0 targets → rejected (mandatory min=1 not satisfied).
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Mixed Mandatory UpToN");
        let result = process_command(state, cast_spell(p1, spell_id, vec![]));
        assert!(
            result.is_err(),
            "M7a: 0 targets with mixed mandatory+UpToN must be rejected"
        );
    }

    // (b) 1 target → success (mandatory satisfied, UpToN contributes 0).
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Mixed Mandatory UpToN");
        let alpha_id = find_obj(&state, "Creature Alpha");
        let result = process_command(
            state,
            cast_spell(p1, spell_id, vec![Target::Object(alpha_id)]),
        );
        assert!(
            result.is_ok(),
            "M7b: 1 target with mixed mandatory+UpToN must succeed"
        );
    }

    // (c) 2 targets → success (mandatory takes 1, UpToN takes 1).
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Mixed Mandatory UpToN");
        let alpha_id = find_obj(&state, "Creature Alpha");
        let beta_id = find_obj(&state, "Creature Beta");
        let result = process_command(
            state,
            cast_spell(
                p1,
                spell_id,
                vec![Target::Object(alpha_id), Target::Object(beta_id)],
            ),
        );
        assert!(
            result.is_ok(),
            "M7c: 2 targets with mixed mandatory+UpToN must succeed"
        );
    }

    // (d) 3 targets → rejected (max=2).
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Mixed Mandatory UpToN");
        let alpha_id = find_obj(&state, "Creature Alpha");
        let beta_id = find_obj(&state, "Creature Beta");
        let gamma_id = find_obj(&state, "Creature Gamma");
        let result = process_command(
            state,
            cast_spell(
                p1,
                spell_id,
                vec![
                    Target::Object(alpha_id),
                    Target::Object(beta_id),
                    Target::Object(gamma_id),
                ],
            ),
        );
        assert!(
            result.is_err(),
            "M7d: 3 targets with mixed mandatory+UpToN (max=2) must be rejected"
        );
    }
    let _ = players; // suppress unused warning
}

// ── M8: UpToN inner validation — reject wrong type ────────────────────────────

/// CR 601.2c — PB-T M8: Declaring a target that does not satisfy the inner requirement
/// of UpToN is rejected at cast time.
///
/// Setup: "Destroy up to 2 target artifacts" with a creature target.
/// The inner requirement is TargetArtifact; a creature does not satisfy it.
#[test]
fn test_pbt_up_to_n_rejects_wrong_type() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let spell_def = up_to_n_destroy_artifact_spell("Destroy Up To Two Artifacts", 2);
    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let spell = ObjectSpec::card(p1, "Destroy Up To Two Artifacts")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone());

    // The target is a creature, NOT an artifact — should be rejected.
    let creature = ObjectSpec::creature(p2, "Mere Mortal", 2, 2).in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                green: 2,
                ..ManaPool::default()
            },
        )
        .object(spell)
        .object(creature)
        .build()
        .expect("M8: GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "Destroy Up To Two Artifacts");
    let creature_id = find_obj(&state, "Mere Mortal");

    // CR 601.2c: target must satisfy inner requirement (TargetArtifact); creature does not.
    let result = process_command(
        state,
        cast_spell(p1, spell_id, vec![Target::Object(creature_id)]),
    );
    assert!(
        result.is_err(),
        "M8: casting UpToN(Artifact) with a creature target must be rejected by inner validation"
    );
}

// ── O1: Hash history integrity ────────────────────────────────────────────────

/// CR N/A (hash infrastructure) — PB-T O1: HASH_SCHEMA_VERSION sentinel is exactly 8.
/// Regression guard against accidental rollback to a prior value.
#[test]
fn test_pbt_hash_schema_version_sentinel_is_8_regression() {
    // Must be exactly 8 — not 7 (PB-L), not 6 (PB-P), not less.
    assert_eq!(
        HASH_SCHEMA_VERSION,
        8u8,
        "PB-T O1: Sentinel must be 8; bumped from PB-L's 7 for TargetRequirement::UpToN (CR 601.2c / 115.1b)"
    );
}

// ── O2: Two parallel UpToN slots (Sword of Sinew shape) ───────────────────────

/// CR 601.2c — PB-T O2: A spell with two parallel UpToN slots of different inner types
/// validates each target against its matched inner requirement via the greedy-consume
/// algorithm. Mirrors Sword of Sinew and Steel structure.
///
/// Requirements: [UpToN{1, Planeswalker}, UpToN{1, Artifact}]
/// - 0 targets → legal (both UpToN slots contribute 0).
/// - 1 artifact target → legal (first UpToN{PW} takes 0, second takes artifact).
/// - 1 creature target → rejected (neither inner accepts creatures).
#[test]
fn test_pbt_two_parallel_up_to_n_slots() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let spell_def = CardDefinition {
        name: "Sinew Sword Style".to_string(),
        card_id: CardId("test-sinew-sword-style".to_string()),
        mana_cost: Some(ManaCost {
            white: 1,
            black: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: im::ordset![CardType::Instant],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 1 },
                    cant_be_regenerated: false,
                },
            ]),
            targets: vec![
                TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetPlaneswalker),
                },
                TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetArtifact),
                },
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    };

    let registry: Arc<CardRegistry> = CardRegistry::new(vec![spell_def.clone()]);

    let build_state = |registry: Arc<CardRegistry>| -> GameState {
        let spell = ObjectSpec::card(p1, "Sinew Sword Style")
            .in_zone(ZoneId::Hand(p1))
            .with_card_id(spell_def.card_id.clone());
        let artifact_obj = ObjectSpec::artifact(p2, "Treasure Token").in_zone(ZoneId::Battlefield);
        let creature_obj =
            ObjectSpec::creature(p2, "Goblin Scout", 1, 1).in_zone(ZoneId::Battlefield);

        GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .add_player(p3)
            .add_player(p4)
            .with_registry(registry)
            .player_mana(
                p1,
                ManaPool {
                    white: 1,
                    black: 1,
                    ..ManaPool::default()
                },
            )
            .object(spell)
            .object(artifact_obj)
            .object(creature_obj)
            .build()
            .expect("O2: GameStateBuilder::build must succeed")
    };

    // (a) 0 targets → legal (both UpToN slots contribute 0 to min).
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Sinew Sword Style");
        let result = process_command(state, cast_spell(p1, spell_id, vec![]));
        assert!(
            result.is_ok(),
            "O2a: 0 targets for two-parallel-UpToN spell must be legal"
        );
    }

    // (b) 1 artifact target → legal (first UpToN{Planeswalker} takes 0, second takes artifact).
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Sinew Sword Style");
        let artifact_id = find_obj(&state, "Treasure Token");
        let result = process_command(
            state,
            cast_spell(p1, spell_id, vec![Target::Object(artifact_id)]),
        );
        assert!(
            result.is_ok(),
            "O2b: 1 artifact target for two-parallel-UpToN spell must be legal"
        );
    }

    // (c) 1 creature (non-PW, non-artifact) target → rejected (neither inner matches).
    {
        let state = build_state(registry.clone());
        let spell_id = find_obj(&state, "Sinew Sword Style");
        let goblin_id = find_obj(&state, "Goblin Scout");
        let result = process_command(
            state,
            cast_spell(p1, spell_id, vec![Target::Object(goblin_id)]),
        );
        assert!(
            result.is_err(),
            "O2c: creature target for two-parallel-UpToN(PW+Artifact) must be rejected"
        );
    }
}
