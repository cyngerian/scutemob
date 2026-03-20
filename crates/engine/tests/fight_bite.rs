//! Fight and Bite effect tests (CR 701.14).
//!
//! Fight (CR 701.14a): Two creatures each deal damage equal to their power to the other.
//! Bite (informal): One creature deals damage equal to its power to another.
//!
//! Key rules verified:
//! - CR 701.14a: Each creature deals its power as damage to the other.
//! - CR 701.14b: If either creature is no longer on the battlefield or not a creature,
//!   neither deals damage (all-or-nothing for Fight; source check for Bite).
//! - CR 701.14c: Self-fight → creature deals 2× its power to itself.
//! - CR 701.14d: Fight/Bite damage is non-combat damage (does not trigger combat damage
//!   triggers; deathtouch/lifelink/infect still apply).
//! - Deathtouch applies to fight damage (kills even with 1 damage).
//! - Lifelink applies to fight/bite damage.
//! - Bite is one-sided: only the source deals damage; target does not deal damage back.

use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardEffectTarget, CardId, CardRegistry,
    CardType, Command, Effect, GameEvent, GameState, GameStateBuilder, KeywordAbility, ManaCost,
    ObjectId, ObjectSpec, PlayerId, Step, TargetController, TargetFilter, TargetRequirement,
    TypeLine, ZoneId,
};

// ── Helper functions ──────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn find_object_on_battlefield(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == ZoneId::Battlefield)
        .map(|(id, _)| *id)
}

fn life_total(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(&player)
        .map(|ps| ps.life_total)
        .unwrap_or_else(|| panic!("player {:?} not found", player))
}

/// Pass priority for all listed players once.
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

/// Cast an instant/sorcery spell from hand with two creature targets.
fn cast_spell_two_targets(
    state: GameState,
    caster: PlayerId,
    spell_id: ObjectId,
    target0: ObjectId,
    target1: ObjectId,
) -> GameState {
    let mut state = state;
    // Fund with colorless mana.
    state
        .players
        .get_mut(&caster)
        .unwrap()
        .mana_pool
        .add(mtg_engine::ManaColor::Colorless, 10);
    state.turn.priority_holder = Some(caster);

    let (state, _) = process_command(
        state,
        Command::CastSpell {
            player: caster,
            card: spell_id,
            targets: vec![
                mtg_engine::Target::Object(target0),
                mtg_engine::Target::Object(target1),
            ],
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
    .unwrap_or_else(|e| panic!("CastSpell (two targets) failed: {:?}", e));
    state
}

// ── Fight spell card definitions ──────────────────────────────────────────────

/// "Test Fight" instant: target creature you control fights target creature you don't control.
fn fight_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-fight-spell".to_string()),
        name: "Test Fight Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Target creature fights target creature.".to_string(),
        power: None,
        toughness: None,
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Fight {
                attacker: CardEffectTarget::DeclaredTarget { index: 0 },
                defender: CardEffectTarget::DeclaredTarget { index: 1 },
            },
            targets: vec![
                TargetRequirement::TargetCreature,
                TargetRequirement::TargetCreature,
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// "Test Bite" instant: target creature you control bites target creature you don't control.
fn bite_spell_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("test-bite-spell".to_string()),
        name: "Test Bite Spell".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Target creature deals damage equal to its power to target creature."
            .to_string(),
        power: None,
        toughness: None,
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Bite {
                source: CardEffectTarget::DeclaredTarget { index: 0 },
                target: CardEffectTarget::DeclaredTarget { index: 1 },
            },
            targets: vec![
                TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                }),
                TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                }),
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

// ── Tests: Fight ──────────────────────────────────────────────────────────────

#[test]
/// CR 701.14a — Fight basic: two creatures fight, each takes damage equal to the
/// other's power. A 3/3 fights a 2/4 — 3/3 marks 2 damage, 2/4 marks 3 damage.
fn test_fight_basic() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fight_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Fight Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-fight-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let attacker = ObjectSpec::creature(p1, "P1 Attacker", 3, 3).in_zone(ZoneId::Battlefield);
    let defender = ObjectSpec::creature(p2, "P2 Defender", 2, 4).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(attacker)
        .object(defender)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Fight Spell");
    let att_id = find_object(&state, "P1 Attacker");
    let def_id = find_object(&state, "P2 Defender");

    let state = cast_spell_two_targets(state, p1, spell_id, att_id, def_id);

    // Resolve: both players pass priority.
    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.14a: P1 Attacker (3 power) → P2 Defender takes 3 damage.
    let def_dmg = state
        .objects
        .get(&def_id)
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(
        def_dmg, 3,
        "CR 701.14a: P2 Defender (2/4) should have 3 damage marked from 3-power attacker"
    );

    // CR 701.14a: P2 Defender (2 power) → P1 Attacker takes 2 damage.
    let att_dmg = state
        .objects
        .get(&att_id)
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(
        att_dmg, 2,
        "CR 701.14a: P1 Attacker (3/3) should have 2 damage marked from 2-power defender"
    );

    // DamageDealt events emitted for both directions.
    let dmg_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DamageDealt { .. }))
        .collect();
    assert_eq!(
        dmg_events.len(),
        2,
        "CR 701.14a: Two DamageDealt events should fire (one per direction)"
    );
}

#[test]
/// CR 701.14a + SBA 704.5g — A 5/5 fights a 2/2.
/// 2/2 receives 5 lethal damage and dies. 5/5 receives 2 damage and survives.
fn test_fight_one_dies() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fight_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Fight Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-fight-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let big = ObjectSpec::creature(p1, "Big Creature", 5, 5).in_zone(ZoneId::Battlefield);
    let small = ObjectSpec::creature(p2, "Small Creature", 2, 2).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(big)
        .object(small)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Fight Spell");
    let big_id = find_object(&state, "Big Creature");
    let small_id = find_object(&state, "Small Creature");

    let state = cast_spell_two_targets(state, p1, spell_id, big_id, small_id);

    // Resolve: pass priority to resolve the spell and trigger SBA checks.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.14a + SBA 704.5g: Small Creature took 5 lethal damage; it should be in the graveyard.
    let small_on_bf = find_object_on_battlefield(&state, "Small Creature");
    assert!(
        small_on_bf.is_none(),
        "CR 701.14a + SBA 704.5g: Small Creature (2/2) should die from 5 damage"
    );

    // Big Creature took 2 damage but has 5 toughness — should still be alive.
    let big_on_bf = find_object_on_battlefield(&state, "Big Creature");
    assert!(
        big_on_bf.is_some(),
        "CR 701.14a: Big Creature (5/5) should survive 2 damage from Small Creature"
    );
    let big_dmg = big_on_bf
        .and_then(|id| state.objects.get(&id))
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(
        big_dmg, 2,
        "CR 701.14a: Big Creature should have 2 damage marked"
    );
}

#[test]
/// CR 701.14a + SBA 704.5g — A 3/3 fights a 3/3. Both take 3 damage and die.
fn test_fight_both_die() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fight_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Fight Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-fight-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let creature_a = ObjectSpec::creature(p1, "Creature A", 3, 3).in_zone(ZoneId::Battlefield);
    let creature_b = ObjectSpec::creature(p2, "Creature B", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(creature_a)
        .object(creature_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Fight Spell");
    let a_id = find_object(&state, "Creature A");
    let b_id = find_object(&state, "Creature B");

    let state = cast_spell_two_targets(state, p1, spell_id, a_id, b_id);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Both 3/3 creatures took 3 lethal damage and should be in the graveyard.
    let a_on_bf = find_object_on_battlefield(&state, "Creature A");
    let b_on_bf = find_object_on_battlefield(&state, "Creature B");
    assert!(
        a_on_bf.is_none(),
        "CR 701.14a + SBA 704.5g: Creature A (3/3) should die from 3 damage"
    );
    assert!(
        b_on_bf.is_none(),
        "CR 701.14a + SBA 704.5g: Creature B (3/3) should die from 3 damage"
    );
}

#[test]
/// CR 701.14c — Self-fight: a creature fights itself. It deals damage equal to
/// twice its power to itself. A 3/4 fighting itself takes 6 damage (2 × 3) and dies.
fn test_fight_self() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fight_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Fight Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-fight-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    // 3/4: taking 6 damage (2×3) exceeds toughness, so it should die.
    let creature = ObjectSpec::creature(p1, "Self Fighter", 3, 4).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Fight Spell");
    let creature_id = find_object(&state, "Self Fighter");

    // Cast the fight spell with the same creature as both targets.
    let state = cast_spell_two_targets(state, p1, spell_id, creature_id, creature_id);
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.14c: 3/4 creature receives 2×3=6 damage. SBA: 6 >= 4 toughness → dies.
    let on_bf = find_object_on_battlefield(&state, "Self Fighter");
    assert!(
        on_bf.is_none(),
        "CR 701.14c: Self Fighter (3/4) should die from 6 self-fight damage (2×3 power)"
    );
}

#[test]
/// CR 701.14d — Fight damage is non-combat damage. It does NOT trigger "whenever
/// this creature deals combat damage" abilities. We verify by checking that player
/// life totals are unchanged (fight is creature-vs-creature, not vs player).
fn test_fight_non_combat_damage() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fight_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Fight Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-fight-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let creature_a = ObjectSpec::creature(p1, "Creature A", 3, 5).in_zone(ZoneId::Battlefield);
    let creature_b = ObjectSpec::creature(p2, "Creature B", 3, 5).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(creature_a)
        .object(creature_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let p1_life_before = life_total(&state, p1);
    let p2_life_before = life_total(&state, p2);

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Fight Spell");
    let a_id = find_object(&state, "Creature A");
    let b_id = find_object(&state, "Creature B");

    let state = cast_spell_two_targets(state, p1, spell_id, a_id, b_id);
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.14d: Fight damage goes to creatures, not players — life totals unchanged.
    assert_eq!(
        life_total(&state, p1),
        p1_life_before,
        "CR 701.14d: Fight damage is creature-to-creature; P1 life unchanged"
    );
    assert_eq!(
        life_total(&state, p2),
        p2_life_before,
        "CR 701.14d: Fight damage is creature-to-creature; P2 life unchanged"
    );

    // Both creatures took damage (fight was non-combat but still happened).
    let a_dmg = state
        .objects
        .get(&a_id)
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    let b_dmg = state
        .objects
        .get(&b_id)
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(a_dmg, 3, "Creature A should have 3 damage marked");
    assert_eq!(b_dmg, 3, "Creature B should have 3 damage marked");
}

#[test]
/// CR 701.14d — Deathtouch applies to fight (non-combat) damage. A 1/1 deathtouch
/// creature fights a 5/5. The 5/5 dies from 1 deathtouch damage; 1/1 takes 5 damage.
fn test_fight_deathtouch() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fight_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Fight Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-fight-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    // 1/1 with Deathtouch: any damage it deals is treated as lethal (SBA 704.5h).
    let deathtouch_creature = ObjectSpec::creature(p1, "Deathtouch Creature", 1, 1)
        .with_keyword(KeywordAbility::Deathtouch)
        .in_zone(ZoneId::Battlefield);
    let beefy = ObjectSpec::creature(p2, "Beefy Creature", 5, 5).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(deathtouch_creature)
        .object(beefy)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Fight Spell");
    let dt_id = find_object(&state, "Deathtouch Creature");
    let beefy_id = find_object(&state, "Beefy Creature");

    let state = cast_spell_two_targets(state, p1, spell_id, dt_id, beefy_id);
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.2b + SBA 704.5h: Deathtouch damage is lethal regardless of toughness.
    // Beefy Creature (5/5) took 1 deathtouch damage → SBA marks it for death.
    let beefy_on_bf = find_object_on_battlefield(&state, "Beefy Creature");
    assert!(
        beefy_on_bf.is_none(),
        "CR 702.2b + SBA 704.5h: Beefy Creature should die from 1 deathtouch fight damage"
    );

    // Deathtouch Creature (1/1) took 5 damage ≥ 1 toughness → also dies.
    let dt_on_bf = find_object_on_battlefield(&state, "Deathtouch Creature");
    assert!(
        dt_on_bf.is_none(),
        "CR 701.14a + SBA 704.5g: Deathtouch Creature (1/1) takes 5 damage and dies"
    );
}

#[test]
/// CR 701.14d — Lifelink applies to fight (non-combat) damage.
/// A 4/4 lifelink creature fights a 3/3. The lifelink controller gains 4 life.
fn test_fight_lifelink() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fight_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Fight Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-fight-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let lifelink_creature = ObjectSpec::creature(p1, "Lifelink Creature", 4, 4)
        .with_keyword(KeywordAbility::Lifelink)
        .in_zone(ZoneId::Battlefield);
    let enemy = ObjectSpec::creature(p2, "Enemy Creature", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(lifelink_creature)
        .object(enemy)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let p1_life_before = life_total(&state, p1);

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Fight Spell");
    let ll_id = find_object(&state, "Lifelink Creature");
    let enemy_id = find_object(&state, "Enemy Creature");

    let state = cast_spell_two_targets(state, p1, spell_id, ll_id, enemy_id);
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.15b: Lifelink — controller gains life equal to damage dealt.
    // Lifelink Creature deals 4 damage (its power) → P1 gains 4 life.
    assert_eq!(
        life_total(&state, p1),
        p1_life_before + 4,
        "CR 702.15b: Lifelink creature fighting should gain P1 life equal to damage dealt"
    );
}

// ── Tests: Bite ───────────────────────────────────────────────────────────────

#[test]
/// CR 701.14 (one-sided) — Bite basic: source creature deals its power as damage
/// to target; target does NOT deal damage back.
/// A 4/2 bites a 3/8. The 3/8 takes 4 damage. The 4/2 takes NO damage.
/// (Using 3/8 so the target survives and damage_marked can be checked.)
fn test_bite_basic() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bite_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Bite Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-bite-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let source = ObjectSpec::creature(p1, "Bite Source", 4, 2).in_zone(ZoneId::Battlefield);
    // Use 3/8 so the target survives 4 damage and we can check damage_marked directly.
    let target = ObjectSpec::creature(p2, "Bite Target", 3, 8).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(source)
        .object(target)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Bite Spell");
    let src_id = find_object(&state, "Bite Source");
    let tgt_id = find_object(&state, "Bite Target");

    let state = cast_spell_two_targets(state, p1, spell_id, src_id, tgt_id);
    let (state, events) = pass_all(state, &[p1, p2]);

    // Target takes damage equal to source's power.
    let tgt_dmg = state
        .objects
        .get(&tgt_id)
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(
        tgt_dmg, 4,
        "Bite: Bite Target (3/8) should take 4 damage (source's 4 power)"
    );

    // Source takes NO damage (Bite is one-sided).
    let src_dmg = state
        .objects
        .get(&src_id)
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(
        src_dmg, 0,
        "Bite: Bite Source (4/2) should take ZERO damage — Bite is one-sided"
    );

    // Only one DamageDealt event should fire (not two — Bite is one-sided).
    let dmg_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DamageDealt { .. }))
        .count();
    assert_eq!(
        dmg_events, 1,
        "Bite: Exactly one DamageDealt event should fire (source deals to target only)"
    );
}

#[test]
/// CR 701.14 (one-sided) — Bite with zero-power source deals no damage.
/// A 0/4 creature bites a 3/3. Nothing happens.
fn test_bite_zero_power() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bite_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Bite Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-bite-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let source = ObjectSpec::creature(p1, "Zero Power", 0, 4).in_zone(ZoneId::Battlefield);
    let target = ObjectSpec::creature(p2, "Bite Target", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(source)
        .object(target)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Bite Spell");
    let src_id = find_object(&state, "Zero Power");
    let tgt_id = find_object(&state, "Bite Target");

    let state = cast_spell_two_targets(state, p1, spell_id, src_id, tgt_id);
    let (state, _) = pass_all(state, &[p1, p2]);

    // Source has 0 power → 0 damage dealt.
    let tgt_dmg = state
        .objects
        .get(&tgt_id)
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(
        tgt_dmg, 0,
        "Bite: 0-power source should deal 0 damage to target"
    );

    let tgt_on_bf = find_object_on_battlefield(&state, "Bite Target");
    assert!(
        tgt_on_bf.is_some(),
        "Bite Target (3/3) should survive zero-power bite"
    );
}

#[test]
/// CR 701.14d — Lifelink applies to Bite damage.
/// A 3/3 lifelink creature bites a 5/5. Lifelink controller gains 3 life.
fn test_bite_lifelink() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bite_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Bite Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-bite-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let source = ObjectSpec::creature(p1, "Lifelink Biter", 3, 3)
        .with_keyword(KeywordAbility::Lifelink)
        .in_zone(ZoneId::Battlefield);
    let target = ObjectSpec::creature(p2, "Big Target", 5, 5).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(source)
        .object(target)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    let p1_life_before = life_total(&state, p1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Bite Spell");
    let src_id = find_object(&state, "Lifelink Biter");
    let tgt_id = find_object(&state, "Big Target");

    let state = cast_spell_two_targets(state, p1, spell_id, src_id, tgt_id);
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 702.15b: Lifelink — controller of the source gains life equal to bite damage.
    assert_eq!(
        life_total(&state, p1),
        p1_life_before + 3,
        "CR 702.15b: Lifelink Biter (3/3) bite should gain P1 3 life"
    );

    // Big Target took 3 damage.
    let tgt_dmg = state
        .objects
        .get(&tgt_id)
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(tgt_dmg, 3, "Big Target should have 3 damage marked");
}

#[test]
/// CR 701.14b (analog) — Bite: source creature not on battlefield at resolution.
/// If the source is removed before the spell resolves, no damage is dealt.
/// We test this by having the source creature die (SBA) before the spell resolves.
/// Since we can't easily remove a creature mid-resolution with this harness,
/// we verify that Bite with a source that has been moved to the graveyard is a no-op.
/// This is a structural test of the is_creature_on_battlefield guard in the dispatch.
fn test_bite_source_creature_killed_before_resolution() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![bite_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Bite Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-bite-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let source = ObjectSpec::creature(p1, "Bite Source", 4, 2).in_zone(ZoneId::Battlefield);
    let target = ObjectSpec::creature(p2, "Bite Target", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(source)
        .object(target)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Bite Spell");
    let src_id = find_object(&state, "Bite Source");
    let tgt_id = find_object(&state, "Bite Target");

    // Cast the bite spell onto the stack.
    let mut state = cast_spell_two_targets(state, p1, spell_id, src_id, tgt_id);

    // Before resolution: manually move the source creature to the graveyard
    // (simulating it being killed in response).
    let graveyard = ZoneId::Graveyard(p1);
    state
        .move_object_to_zone(src_id, graveyard)
        .expect("move source to graveyard");

    // Resolve: pass priority.
    let (state, _) = pass_all(state, &[p1, p2]);

    // CR 701.14b analog: source is not on the battlefield → no damage dealt.
    let tgt_dmg = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Bite Target")
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(
        tgt_dmg, 0,
        "CR 701.14b analog: Bite with source off battlefield should deal no damage"
    );
}

#[test]
/// CR 701.14b — Fight: one creature is removed from the battlefield in response.
/// Neither creature deals damage (all-or-nothing).
fn test_fight_creature_left_battlefield() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![fight_spell_def()]);

    let spell = ObjectSpec::card(p1, "Test Fight Spell")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(CardId("test-fight-spell".to_string()))
        .with_types(vec![CardType::Instant])
        .with_mana_cost(ManaCost {
            generic: 1,
            ..Default::default()
        });
    let creature_a = ObjectSpec::creature(p1, "Creature A", 3, 3).in_zone(ZoneId::Battlefield);
    let creature_b = ObjectSpec::creature(p2, "Creature B", 3, 3).in_zone(ZoneId::Battlefield);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(creature_a)
        .object(creature_b)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Test Fight Spell");
    let a_id = find_object(&state, "Creature A");
    let b_id = find_object(&state, "Creature B");

    // Cast fight spell onto the stack.
    let mut state = cast_spell_two_targets(state, p1, spell_id, a_id, b_id);

    // Before resolution: remove Creature B from the battlefield (bounced/exiled/killed).
    let graveyard = ZoneId::Graveyard(p2);
    state
        .move_object_to_zone(b_id, graveyard)
        .expect("move Creature B to graveyard");

    // Resolve the fight spell.
    let (state, events) = pass_all(state, &[p1, p2]);

    // CR 701.14b: Creature B left the battlefield → neither creature fights.
    // Creature A should NOT have taken any damage.
    let a_on_bf = find_object_on_battlefield(&state, "Creature A");
    assert!(
        a_on_bf.is_some(),
        "Creature A should still be on battlefield"
    );
    let a_dmg = a_on_bf
        .and_then(|id| state.objects.get(&id))
        .map(|o| o.damage_marked)
        .unwrap_or(0);
    assert_eq!(
        a_dmg, 0,
        "CR 701.14b: Creature A should take 0 damage when Creature B left before fight"
    );

    // No DamageDealt events should have fired.
    let dmg_events = events
        .iter()
        .filter(|e| matches!(e, GameEvent::DamageDealt { .. }))
        .count();
    assert_eq!(
        dmg_events, 0,
        "CR 701.14b: No DamageDealt events when one fight target left the battlefield"
    );
}
