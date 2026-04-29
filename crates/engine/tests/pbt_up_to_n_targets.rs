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

/// CR N/A (hash infrastructure) — PB-T M4: HASH_SCHEMA_VERSION is 10 (PB-CC-B bump). Three
/// distinct UpToN variants hash to distinct values, confirming the new discriminant-17 arm
/// is reached and the count/inner fields contribute to the hash.
#[test]
fn test_pbt_hash_schema_version_is_10() {
    use blake3::Hasher;
    use mtg_engine::state::hash::HashInto;

    // CR N/A — sentinel must be 10 (PB-CC-B bump from PB-SFT's 9 for
    // TargetFilter.has_counter_type field (CR 121 counter presence predicate)).
    assert_eq!(
        HASH_SCHEMA_VERSION, 10u8,
        "PB-CC-B: HASH_SCHEMA_VERSION must be 10 (bump from PB-SFT's 9 for \
         TargetFilter.has_counter_type counter presence predicate)"
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

// ── M5: Partial target declaration resolves for declared targets ───────────────

/// CR 601.2c — PB-T M5: A spell declared with fewer than max UpToN targets
/// (partial declaration at cast time) resolves for the declared targets without
/// fizzling. This verifies the foundational partial-declaration semantics:
/// declaring 1 of up-to-2 targets is legal and the declared target is affected.
///
/// Setup: "Tap up to 2 permanents" cast with 1 target. The 1 declared target is tapped.
/// The spell does not fizzle (CR 608.2b: fizzle only if ALL targets become illegal).
///
/// Note: this test exercises partial *declaration at cast time* (player chose 1 of 2),
/// NOT zone-change partial fizzle (one target leaves mid-stack). See M9 for the
/// genuine zone-change partial-fizzle scenario.
#[test]
fn test_pbt_up_to_n_partial_target_declaration_resolves() {
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

    // CR 601.2c: Declare 1 of up-to-2 targets (partial, legal).
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

/// CR N/A (hash infrastructure) — PB-T O1: HASH_SCHEMA_VERSION sentinel is exactly 10.
/// Regression guard against accidental rollback to a prior value.
#[test]
fn test_pbt_hash_schema_version_sentinel_is_10_regression() {
    // Must be exactly 10 (PB-CC-B bump from PB-SFT's 9 for TargetFilter.has_counter_type).
    assert_eq!(
        HASH_SCHEMA_VERSION, 10u8,
        "PB-CC-B: Sentinel must be 10; bumped from PB-SFT's 9 for \
         TargetFilter.has_counter_type (CR 121 counter presence predicate)"
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

// ── M9: Zone-change partial fizzle (CR 608.2b) ────────────────────────────────

/// CR 608.2b / CR 400.7 — PB-T M9: When an UpToN spell is declared with N targets and
/// one target becomes illegal between cast time and resolution (zone change), only the
/// illegal target is dropped; the surviving legal target is still affected.
///
/// This is the genuine zone-change partial-fizzle test required by CR 608.2b:
/// "If some targets are no longer legal when the spell tries to resolve, the illegal
/// targets are simply ignored. Remaining legal targets are still affected."
///
/// Setup:
/// - P1 has "Tap Up To Two" (UpToN{2, Creature}) in hand.
/// - P2 has "Destroy Creature" (mandatory 1 creature target) in hand.
/// - Creature A (P2's) and Creature B (P2's) are on the battlefield.
///
/// Sequence:
/// 1. P1 casts "Tap Up To Two" targeting Creature A AND Creature B.
/// 2. P1 passes → P2 casts "Destroy Creature" targeting Creature A (stack: [Destroy, TapTwo]).
/// 3. All pass → Destroy resolves; Creature A → graveyard (zone change, new object CR 400.7).
/// 4. All pass → "Tap Up To Two" resolves; A is no longer on battlefield (illegal) — dropped;
///    Creature B is still legal — gets tapped.
///
/// Assert: Creature A is in graveyard; Creature B is tapped on the battlefield.
#[test]
fn test_pbt_up_to_n_partial_fizzle_on_zone_change() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let tap_spell_def = up_to_n_tap_permanent_spell("Tap Up To Two", 2);
    let destroy_spell_def = mandatory_target_destroy_creature_spell("Destroy Creature");
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![tap_spell_def.clone(), destroy_spell_def.clone()]);

    let tap_spell = ObjectSpec::card(p1, "Tap Up To Two")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(tap_spell_def.card_id.clone());

    let destroy_spell = ObjectSpec::card(p2, "Destroy Creature")
        .in_zone(ZoneId::Hand(p2))
        .with_card_id(destroy_spell_def.card_id.clone())
        .with_types(vec![CardType::Instant]);

    let creature_a = ObjectSpec::creature(p2, "Creature A", 2, 2).in_zone(ZoneId::Battlefield);
    let creature_b = ObjectSpec::creature(p2, "Creature B", 3, 3).in_zone(ZoneId::Battlefield);

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
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..ManaPool::default()
            },
        )
        .object(tap_spell)
        .object(destroy_spell)
        .object(creature_a)
        .object(creature_b)
        .build()
        .expect("M9: GameStateBuilder::build must succeed");

    let tap_id = find_obj(&state, "Tap Up To Two");
    let destroy_id = find_obj(&state, "Destroy Creature");
    let crea_a_id = find_obj(&state, "Creature A");
    let crea_b_id = find_obj(&state, "Creature B");

    // Step 1: P1 casts "Tap Up To Two" targeting BOTH Creature A and Creature B.
    let (state, _) = process_command(
        state,
        cast_spell(
            p1,
            tap_id,
            vec![Target::Object(crea_a_id), Target::Object(crea_b_id)],
        ),
    )
    .expect("M9: P1 casts Tap Up To Two targeting 2 creatures");

    // Step 2: P1 passes priority; P2 casts "Destroy Creature" targeting A in response.
    let (state, _) = process_command(state, Command::PassPriority { player: p1 })
        .expect("M9: P1 passes priority after casting");
    let (state, _) = process_command(
        state,
        cast_spell(p2, destroy_id, vec![Target::Object(crea_a_id)]),
    )
    .expect("M9: P2 casts Destroy Creature targeting Creature A");

    // Step 3: All players pass priority → "Destroy Creature" resolves; Creature A dies.
    let (state, _) = pass_all(state, &players);

    // Verify Creature A is dead after Destroy resolves (zone change per CR 400.7).
    assert!(
        obj_in_graveyard(&state, "Creature A", p2),
        "M9: Creature A must be in graveyard after Destroy Creature resolves"
    );
    assert!(
        obj_on_battlefield(&state, "Creature B"),
        "M9: Creature B must still be on battlefield"
    );

    // Step 4: All players pass priority → "Tap Up To Two" resolves.
    // CR 608.2b: Creature A is no longer on battlefield → illegal target → dropped.
    // Creature B is still legal → tapped.
    let (state, _) = pass_all(state, &players);

    // Creature A remains in the graveyard (UpToN skipped the illegal target).
    assert!(
        obj_in_graveyard(&state, "Creature A", p2),
        "M9: Creature A must remain in graveyard after Tap Up To Two resolves (partial fizzle)"
    );
    // Creature B must be tapped (surviving legal target per CR 608.2b).
    let creature_b_obj = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Creature B")
        .expect("M9: Creature B must still exist on battlefield after partial fizzle");
    assert!(
        creature_b_obj.status.tapped,
        "M9: surviving legal target (Creature B) must be tapped per CR 608.2b"
    );
}

// ── M10: Out-of-slot-order declaration succeeds (E1 regression guard) ─────────

/// CR 601.2c — PB-T M10: When an UpToN spell has two parallel UpToN slots
/// `[UpToN{1, Planeswalker}, UpToN{1, Artifact}]` and the player declares targets
/// in reverse slot order (`[artifact, planeswalker]`), the two-pass best-fit
/// validator correctly assigns each target to its matching slot and accepts the cast.
///
/// This is the regression test for E1 (greedy-consume rejected out-of-order declarations).
/// Per CR 601.2c, target declaration order is NOT required to match slot order.
#[test]
fn test_pbt_up_to_n_reverse_order_declaration_succeeds() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let spell_def = CardDefinition {
        name: "Reverse Order Spell".to_string(),
        card_id: CardId("test-reverse-order-spell".to_string()),
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
            // Slot order: [UpToN{Planeswalker}, UpToN{Artifact}]
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

    let spell = ObjectSpec::card(p1, "Reverse Order Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(spell_def.card_id.clone());

    let artifact_obj = ObjectSpec::artifact(p2, "Gold Token").in_zone(ZoneId::Battlefield);
    // Planeswalker: a permanent with Planeswalker card type.
    let planeswalker_obj = ObjectSpec::card(p2, "Test Walker")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Planeswalker]);

    let state = GameStateBuilder::new()
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
        .object(planeswalker_obj)
        .build()
        .expect("M10: GameStateBuilder::build must succeed");

    let spell_id = find_obj(&state, "Reverse Order Spell");
    let artifact_id = find_obj(&state, "Gold Token");
    let pw_id = find_obj(&state, "Test Walker");

    // CR 601.2c: declare [artifact, planeswalker] — REVERSE of slot order [PW, Artifact].
    // Two-pass best-fit: Pass 2 assigns artifact → UpToN{Artifact} slot, PW → UpToN{PW} slot.
    let result = process_command(
        state,
        cast_spell(
            p1,
            spell_id,
            vec![Target::Object(artifact_id), Target::Object(pw_id)],
        ),
    );
    assert!(
        result.is_ok(),
        "M10: reverse-order declaration [artifact, planeswalker] for [UpToN{{PW}}, UpToN{{Artifact}}] must succeed per CR 601.2c"
    );
}

// ── O3: Card integration test (Force of Vigor) ────────────────────────────────

/// CR 601.2c — PB-T O3: Integration test using the real Force of Vigor card
/// definition from the registry. "Destroy up to two target artifacts and/or
/// enchantments." Verifies that the real card def's UpToN targeting validates
/// and resolves correctly with 1 of 2 targets (partial declaration).
///
/// This is a smoke test for PB-T card def regressions: ensures at least one real
/// card def exercises the UpToN path end-to-end (cast → validate → resolve).
#[test]
fn test_pbt_force_of_vigor_card_integration() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);
    let players = [p1, p2, p3, p4];

    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![mtg_engine::cards::defs::force_of_vigor::card()]);

    let fov_spell = ObjectSpec::card(p1, "Force of Vigor")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("force-of-vigor".to_string()));

    let artifact_a = ObjectSpec::artifact(p2, "Artifact Alpha").in_zone(ZoneId::Battlefield);
    let artifact_b = ObjectSpec::artifact(p2, "Artifact Beta").in_zone(ZoneId::Battlefield);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 2,
                green: 2,
                ..ManaPool::default()
            },
        )
        .object(fov_spell)
        .object(artifact_a)
        .object(artifact_b)
        .build()
        .expect("O3: GameStateBuilder::build must succeed");

    let fov_id = find_obj(&state, "Force of Vigor");
    let art_a_id = find_obj(&state, "Artifact Alpha");

    // CR 601.2c: Cast Force of Vigor with 1 of up-to-2 targets (partial declaration).
    let (state, _) = process_command(
        state,
        cast_spell(p1, fov_id, vec![Target::Object(art_a_id)]),
    )
    .expect("O3: Force of Vigor with 1 target must be accepted by UpToN{2, artifact/enchantment}");

    let (state, _) = pass_all(state, &players);

    // CR 608.2b: only the declared target A is destroyed; B is untouched.
    assert!(
        obj_in_graveyard(&state, "Artifact Alpha", p2),
        "O3: declared target (Artifact Alpha) must be destroyed by Force of Vigor"
    );
    assert!(
        obj_on_battlefield(&state, "Artifact Beta"),
        "O3: undeclared target (Artifact Beta) must remain on battlefield"
    );
}
