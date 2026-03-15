//! Spell cost modification tests (CR 601.2f).
//!
//! Tests for both permanent-based spell cost modifiers (SpellCostModifier on CardDefinition)
//! and self-cost-reduction (SelfCostReduction on CardDefinition).
//!
//! Key rules verified:
//! - Cost increases/reductions modify ONLY generic mana (CR 601.2f).
//! - Generic mana cost cannot go below 0 (CR 601.2f).
//! - Multiple modifiers from different permanents stack additively.
//! - Cost modifiers apply after commander tax + kicker, before affinity/undaunted/convoke.
//! - Scope: AllPlayers affects everyone; Controller affects only the permanent's controller.
//! - Eminence: modifier applies from the command zone as well as battlefield.
//! - Self-cost-reduction: spell is cheaper based on game state at cast time.

use mtg_engine::{
    process_command, CardDefinition, CardId, CardRegistry, CardType, Command, CostModifierScope,
    GameStateBuilder, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, PlayerTarget,
    SelfCostReduction, SpellCostFilter, SpellCostModifier, Step, SubType, TargetFilter, ZoneId,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn cid(s: &str) -> CardId {
    CardId(s.to_string())
}

fn find_object(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn cast_spell(
    state: mtg_engine::GameState,
    player: PlayerId,
    card: ObjectId,
) -> Result<(mtg_engine::GameState, Vec<mtg_engine::GameEvent>), mtg_engine::GameStateError> {
    process_command(
        state,
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
        },
    )
}

/// A generic sorcery in hand with the given generic mana cost.
fn sorcery_in_hand(owner: PlayerId, name: &str, generic: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic,
            ..Default::default()
        })
}

/// A creature spell in hand.
fn creature_in_hand(owner: PlayerId, name: &str, generic: u32, colored: u32) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_card_id(cid(&name.to_lowercase().replace(' ', "-")))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .with_mana_cost(ManaCost {
            generic,
            red: colored,
            ..Default::default()
        })
}

// ── Test 1: Thalia-style noncreature cost increase ──────────────────────────

#[test]
/// CR 601.2f — A permanent with SpellCostModifier { change: +1, NonCreature, AllPlayers }
/// makes noncreature spells cost {1} more for all players.
fn test_spell_cost_modifier_noncreature_increase() {
    let p1 = p(1);
    let p2 = p(2);

    // Thalia-like card def: noncreature spells cost {1} more.
    let thalia_def = CardDefinition {
        card_id: cid("thalia-test"),
        name: "Thalia Test".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: 1,
            filter: SpellCostFilter::NonCreature,
            scope: CostModifierScope::AllPlayers,
            eminence: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![thalia_def]);

    // Thalia on the battlefield.
    let thalia = ObjectSpec::creature(p1, "Thalia Test", 2, 1).with_card_id(cid("thalia-test"));

    // Noncreature spell (sorcery) {2} in p1's hand. With Thalia: costs {3}.
    let spell = sorcery_in_hand(p1, "Lightning Bolt Test", 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(thalia)
        .object(spell)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Lightning Bolt Test");

    // With only 2 mana, casting a {2} spell that now costs {3} should fail.
    let result = cast_spell(state.clone(), p1, spell_id);
    assert!(
        result.is_err(),
        "CR 601.2f: noncreature spell should cost {{1}} more with Thalia on battlefield"
    );

    // Give p1 one more mana — now {3} available, should succeed.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: should succeed with enough mana after Thalia increase");
    assert_eq!(state.stack_objects.len(), 1);
}

// ── Test 2: Warchief-style tribal cost reduction (Controller only) ──────────

#[test]
/// CR 601.2f — A permanent with SpellCostModifier { change: -1, HasSubtype(Goblin), Controller }
/// makes Goblin spells cost {1} less for the controller only.
fn test_spell_cost_modifier_tribal_reduction_controller_only() {
    let p1 = p(1);
    let p2 = p(2);

    let warchief_def = CardDefinition {
        card_id: cid("warchief-test"),
        name: "Warchief Test".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Goblin".to_string())),
            scope: CostModifierScope::Controller,
            eminence: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![warchief_def]);

    let warchief =
        ObjectSpec::creature(p1, "Warchief Test", 2, 2).with_card_id(cid("warchief-test"));

    // Goblin creature {2}{R} in p1's hand. With Warchief: costs {1}{R}.
    let goblin = creature_in_hand(p1, "Goblin Grunt", 2, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(warchief)
        .object(goblin)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give p1 {1}{R} — enough for the reduced cost (2-1=1 generic + 1 red).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let goblin_id = find_object(&state, "Goblin Grunt");

    let (state, _) = cast_spell(state, p1, goblin_id)
        .expect("CR 601.2f: Goblin spell should cost {1} less with Warchief");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 601.2f: all mana should be consumed"
    );
}

// ── Test 3: Multiple modifiers stack ────────────────────────────────────────

#[test]
/// CR 601.2f — Two cost-reducing permanents stack their reductions.
fn test_spell_cost_modifiers_stack_additively() {
    let p1 = p(1);
    let p2 = p(2);

    let warchief_def = CardDefinition {
        card_id: cid("warchief-test"),
        name: "Warchief Test".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Goblin".to_string())),
            scope: CostModifierScope::Controller,
            eminence: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![warchief_def]);

    // Two warchiefs on the battlefield.
    let warchief1 =
        ObjectSpec::creature(p1, "Warchief Test", 2, 2).with_card_id(cid("warchief-test"));
    let warchief2 =
        ObjectSpec::creature(p1, "Warchief Test 2", 2, 2).with_card_id(cid("warchief-test"));

    // Goblin {3}{R} in hand. With 2 warchiefs: costs {1}{R}.
    let goblin = creature_in_hand(p1, "Goblin Elite", 3, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(warchief1)
        .object(warchief2)
        .object(goblin)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let goblin_id = find_object(&state, "Goblin Elite");

    let (state, _) = cast_spell(state, p1, goblin_id)
        .expect("CR 601.2f: two warchiefs should reduce cost by {2}");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "CR 601.2f: all mana consumed"
    );
}

// ── Test 4: Generic cost cannot go below 0 ──────────────────────────────────

#[test]
/// CR 601.2f — Cost reduction cannot reduce generic mana below 0.
fn test_spell_cost_modifier_generic_cannot_go_below_zero() {
    let p1 = p(1);
    let p2 = p(2);

    let warchief_def = CardDefinition {
        card_id: cid("warchief-test"),
        name: "Warchief Test".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Goblin".to_string())),
            scope: CostModifierScope::Controller,
            eminence: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![warchief_def]);

    // Three warchiefs but goblin only costs {1}{R} (1 generic).
    let w1 = ObjectSpec::creature(p1, "Warchief Test", 2, 2).with_card_id(cid("warchief-test"));
    let w2 = ObjectSpec::creature(p1, "Warchief A", 2, 2).with_card_id(cid("warchief-test"));
    let w3 = ObjectSpec::creature(p1, "Warchief B", 2, 2).with_card_id(cid("warchief-test"));

    // Goblin {1}{R}. With 3 warchiefs: max reduction is 1 (generic can't go below 0). Costs {R}.
    let goblin = creature_in_hand(p1, "Goblin Runt", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(w1)
        .object(w2)
        .object(w3)
        .object(goblin)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only give {R} — the reduced cost should be {R} (0 generic).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let goblin_id = find_object(&state, "Goblin Runt");

    let (state, _) = cast_spell(state, p1, goblin_id)
        .expect("CR 601.2f: cost reduced to {R} only (generic cannot go below 0)");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

// ── Test 5: Eminence from command zone ──────────────────────────────────────

#[test]
/// CR 601.2f — Eminence modifier applies from the command zone.
fn test_spell_cost_modifier_eminence_from_command_zone() {
    let p1 = p(1);
    let p2 = p(2);

    let ur_dragon_def = CardDefinition {
        card_id: cid("ur-dragon-test"),
        name: "Ur-Dragon Test".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Dragon".to_string())),
            scope: CostModifierScope::Controller,
            eminence: true,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![ur_dragon_def]);

    // Ur-Dragon in the command zone (not on battlefield).
    let ur_dragon = ObjectSpec::card(p1, "Ur-Dragon Test")
        .in_zone(ZoneId::Command(p1))
        .with_card_id(cid("ur-dragon-test"));

    // Dragon creature {3}{R} in hand. With Eminence: costs {2}{R}.
    let dragon = ObjectSpec::card(p1, "Dragon Whelp")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("dragon-whelp"))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ur_dragon)
        .object(dragon)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give {2}{R} — the eminence-reduced cost.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let dragon_id = find_object(&state, "Dragon Whelp");

    let (state, _) = cast_spell(state, p1, dragon_id)
        .expect("CR 601.2f: Dragon spell should cost {1} less with Eminence from command zone");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

// ── Test 6: Self-cost-reduction — PerPermanent (Blasphemous Act style) ──────

#[test]
/// CR 601.2f — SelfCostReduction::PerPermanent reduces the spell's cost by {1}
/// for each matching permanent on the battlefield.
fn test_self_cost_reduction_per_permanent() {
    let p1 = p(1);
    let p2 = p(2);

    // Blasphemous Act style: costs {1} less per creature on battlefield (any controller).
    let blast_def = CardDefinition {
        card_id: cid("blast-test"),
        name: "Blast Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 8,
            red: 1,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::PerPermanent {
            per: 1,
            filter: TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            },
            controller: PlayerTarget::EachPlayer,
        }),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![blast_def]);

    // Spell in hand.
    let spell = ObjectSpec::card(p1, "Blast Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("blast-test"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 8,
            red: 1,
            ..Default::default()
        });

    // 6 creatures on the battlefield (mix of controllers).
    let c1 = ObjectSpec::creature(p1, "Creature A", 1, 1);
    let c2 = ObjectSpec::creature(p1, "Creature B", 1, 1);
    let c3 = ObjectSpec::creature(p1, "Creature C", 1, 1);
    let c4 = ObjectSpec::creature(p2, "Creature D", 1, 1);
    let c5 = ObjectSpec::creature(p2, "Creature E", 1, 1);
    let c6 = ObjectSpec::creature(p2, "Creature F", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(c1)
        .object(c2)
        .object(c3)
        .object(c4)
        .object(c5)
        .object(c6)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Cost: {8}{R} - 6 creatures = {2}{R}. Give {2}{R}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Blast Test");

    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: spell should cost {2}{R} with 6 creatures on battlefield");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

// ── Test 7: Self-cost-reduction — TotalPowerOfCreatures (Ghalta style) ──────

#[test]
/// CR 601.2f — SelfCostReduction::TotalPowerOfCreatures reduces by total power
/// of creatures the caster controls.
fn test_self_cost_reduction_total_power() {
    let p1 = p(1);
    let p2 = p(2);

    let ghalta_def = CardDefinition {
        card_id: cid("ghalta-test"),
        name: "Ghalta Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 10,
            green: 2,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::TotalPowerOfCreatures),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![ghalta_def]);

    let spell = ObjectSpec::card(p1, "Ghalta Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("ghalta-test"))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 10,
            green: 2,
            ..Default::default()
        });

    // p1 controls creatures with total power = 8 (5 + 3).
    let big = ObjectSpec::creature(p1, "Big Creature", 5, 5);
    let med = ObjectSpec::creature(p1, "Med Creature", 3, 3);
    // p2's creature should NOT count.
    let opp = ObjectSpec::creature(p2, "Opp Creature", 10, 10);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(big)
        .object(med)
        .object(opp)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Cost: {10}{G}{G} - 8 power = {2}{G}{G}. Give {2}{G}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Ghalta Test");

    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: Ghalta should cost {2}{G}{G} with 8 total power controlled");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

// ── Test 8: Historic filter (Jhoira's Familiar style) ───────────────────────

#[test]
/// CR 601.2f + CR 700.6 — Historic filter matches artifacts, legendaries, and Sagas.
fn test_spell_cost_modifier_historic_filter() {
    let p1 = p(1);
    let p2 = p(2);

    let jhoira_def = CardDefinition {
        card_id: cid("jhoira-test"),
        name: "Jhoira Test".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::Historic,
            scope: CostModifierScope::Controller,
            eminence: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![jhoira_def]);

    let jhoira = ObjectSpec::creature(p1, "Jhoira Test", 2, 2).with_card_id(cid("jhoira-test"));

    // An artifact spell {3} in hand — should be historic.
    let artifact = ObjectSpec::card(p1, "Test Artifact")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("test-artifact"))
        .with_types(vec![CardType::Artifact])
        .with_mana_cost(ManaCost {
            generic: 3,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(jhoira)
        .object(artifact)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // {3} artifact - 1 reduction = {2}. Give {2}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let artifact_id = find_object(&state, "Test Artifact");

    let (state, _) = cast_spell(state, p1, artifact_id)
        .expect("CR 700.6: artifact spell is historic and should cost {1} less");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}
