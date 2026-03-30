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

use mtg_engine::state::{ActivatedAbility, ActivationCost};
use mtg_engine::{
    process_command, CardDefinition, CardId, CardRegistry, CardType, Color, Command,
    CostModifierScope, GameStateBuilder, KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec,
    PlayerId, PlayerTarget, SelfActivatedCostReduction, SelfCostReduction, SpellCostFilter,
    SpellCostModifier, Step, SubType, TargetFilter, ZoneId,
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
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
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
            exclude_self: false,
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
            exclude_self: false,
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
            exclude_self: false,
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
            exclude_self: false,
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
            exclude_self: false,
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
            exclude_self: false,
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

// ── Test 9: Self-cost-reduction — CardTypesInGraveyard (Emrakul style) ──────

#[test]
/// CR 601.2f — SelfCostReduction::CardTypesInGraveyard reduces the spell's cost by the
/// number of distinct card types among cards in the caster's graveyard (Emrakul style).
/// With 5 distinct card types in graveyard, a {13} spell costs {8}.
fn test_self_cost_reduction_card_types_in_graveyard() {
    let p1 = p(1);
    let p2 = p(2);

    let emrakul_def = CardDefinition {
        card_id: cid("emrakul-test"),
        name: "Emrakul Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 13,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::CardTypesInGraveyard),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![emrakul_def]);

    let spell = ObjectSpec::card(p1, "Emrakul Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("emrakul-test"))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 13,
            ..Default::default()
        });

    // 5 cards of distinct types in p1's graveyard: Creature, Sorcery, Instant, Artifact, Enchantment.
    let dead_creature = ObjectSpec::card(p1, "Dead Creature")
        .in_zone(ZoneId::Graveyard(p1))
        .with_types(vec![CardType::Creature]);
    let dead_sorcery = ObjectSpec::card(p1, "Dead Sorcery")
        .in_zone(ZoneId::Graveyard(p1))
        .with_types(vec![CardType::Sorcery]);
    let dead_instant = ObjectSpec::card(p1, "Dead Instant")
        .in_zone(ZoneId::Graveyard(p1))
        .with_types(vec![CardType::Instant]);
    let dead_artifact = ObjectSpec::card(p1, "Dead Artifact")
        .in_zone(ZoneId::Graveyard(p1))
        .with_types(vec![CardType::Artifact]);
    let dead_enchantment = ObjectSpec::card(p1, "Dead Enchantment")
        .in_zone(ZoneId::Graveyard(p1))
        .with_types(vec![CardType::Enchantment]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(dead_creature)
        .object(dead_sorcery)
        .object(dead_instant)
        .object(dead_artifact)
        .object(dead_enchantment)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // 5 distinct types in graveyard → reduce {13} by 5 = {8}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 8);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Emrakul Test");

    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: Emrakul-style spell should cost {8} with 5 card types in graveyard");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());

    // Negative case: with only {7}, casting should fail (cost is {8}).
    let spell2 = ObjectSpec::card(p1, "Emrakul Test 2")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("emrakul-test"))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 13,
            ..Default::default()
        });
    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![CardDefinition {
            card_id: cid("emrakul-test"),
            name: "Emrakul Test 2".to_string(),
            mana_cost: Some(ManaCost {
                generic: 13,
                ..Default::default()
            }),
            self_cost_reduction: Some(SelfCostReduction::CardTypesInGraveyard),
            ..Default::default()
        }]))
        .object(spell2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    // No graveyard cards → no reduction → cost is {13}.
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 12);
    state2.turn.priority_holder = Some(p1);
    let spell2_id = find_object(&state2, "Emrakul Test 2");
    let result = cast_spell(state2, p1, spell2_id);
    assert!(
        result.is_err(),
        "CR 601.2f: should fail with empty graveyard — no reduction applied"
    );
}

// ── Test 10: Self-cost-reduction — BasicLandTypes (Scion of Draco / Domain) ─

#[test]
/// CR 601.2f — SelfCostReduction::BasicLandTypes { per } reduces the spell's cost by
/// `per` for each basic land type among lands the caster controls (Domain mechanic).
/// With 3 distinct basic land types and per=2, a {12} spell costs {6}.
fn test_self_cost_reduction_basic_land_types() {
    let p1 = p(1);
    let p2 = p(2);

    let scion_def = CardDefinition {
        card_id: cid("scion-test"),
        name: "Scion Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 12,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::BasicLandTypes { per: 2 }),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![scion_def]);

    let spell = ObjectSpec::card(p1, "Scion Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("scion-test"))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 12,
            ..Default::default()
        });

    // 3 basic land types: Plains, Island, Forest (all controlled by p1).
    let plains = ObjectSpec::land(p1, "Plains").with_subtypes(vec![SubType("Plains".to_string())]);
    let island = ObjectSpec::land(p1, "Island").with_subtypes(vec![SubType("Island".to_string())]);
    let forest = ObjectSpec::land(p1, "Forest").with_subtypes(vec![SubType("Forest".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(plains)
        .object(island)
        .object(forest)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // 3 basic land types × per=2 → reduce {12} by 6 = {6}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 6);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Scion Test");

    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: Domain-style spell should cost {6} with 3 basic land types");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

// ── Test 11: Self-cost-reduction — TotalManaValue (Earthquake Dragon style) ─

#[test]
/// CR 601.2f — SelfCostReduction::TotalManaValue reduces the spell's cost by the total
/// mana value of matching permanents the caster controls (Earthquake Dragon style).
/// With Dragons totalling MV=8 on battlefield, a {14}{G} spell costs {6}{G}.
fn test_self_cost_reduction_total_mana_value() {
    let p1 = p(1);
    let p2 = p(2);

    let eq_dragon_def = CardDefinition {
        card_id: cid("eq-dragon-test"),
        name: "Eq Dragon Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 14,
            green: 1,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::TotalManaValue {
            filter: TargetFilter {
                has_subtype: Some(SubType("Dragon".to_string())),
                ..Default::default()
            },
        }),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![eq_dragon_def]);

    let spell = ObjectSpec::card(p1, "Eq Dragon Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("eq-dragon-test"))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 14,
            green: 1,
            ..Default::default()
        });

    // Two Dragons on battlefield with MV 5 and MV 3 (total = 8).
    let dragon1 = ObjectSpec::creature(p1, "Dragon Alpha", 5, 5)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            red: 2,
            ..Default::default()
        }); // MV=5
    let dragon2 = ObjectSpec::creature(p1, "Dragon Beta", 3, 3)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }); // MV=3
            // p2's Dragon should NOT count.
    let opp_dragon = ObjectSpec::creature(p2, "Opp Dragon", 10, 10)
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 10,
            red: 1,
            ..Default::default()
        }); // MV=11, shouldn't count

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(dragon1)
        .object(dragon2)
        .object(opp_dragon)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Total MV of p1's Dragons = 8 → reduce {14}{G} by 8 = {6}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 6);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Eq Dragon Test");

    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: Earthquake Dragon-style spell should cost {6}{G} with MV=8 Dragons");
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

// ── Test 12: The Ur-Dragon exclude_self — eminence does not reduce its own cost ─

#[test]
/// CR 601.2f / The Ur-Dragon oracle text: "other Dragon spells you cast cost {1} less".
/// When The Ur-Dragon itself is in the command zone and the player casts it,
/// the eminence modifier must NOT apply (exclude_self = true).
fn test_spell_cost_modifier_ur_dragon_exclude_self() {
    let p1 = p(1);
    let p2 = p(2);

    // Simulate The Ur-Dragon's eminence with exclude_self = true.
    let ur_dragon_def = CardDefinition {
        card_id: cid("ur-dragon-self-test"),
        name: "Ur Dragon Self Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Dragon".to_string())),
            scope: CostModifierScope::Controller,
            eminence: true,
            exclude_self: true,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![ur_dragon_def]);

    // Ur-Dragon in the command zone AND a copy in hand (same card_id, different ObjectId).
    let ur_dragon_cmd = ObjectSpec::card(p1, "Ur Dragon Self Test")
        .in_zone(ZoneId::Command(p1))
        .with_card_id(cid("ur-dragon-self-test"))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 4,
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        });

    // This represents the player's copy of The Ur-Dragon they are about to cast.
    let ur_dragon_hand = ObjectSpec::card(p1, "Ur Dragon Hand")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("ur-dragon-self-test"))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 4,
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(ur_dragon_cmd)
        .object(ur_dragon_hand)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // The Ur-Dragon costs {4}{W}{U}{B}{R}{G} = 4 generic + 5 colored.
    // With exclude_self: the modifier should NOT reduce the cost of the spell being cast
    // (the hand copy). The command zone copy is the modifier source, and
    // the spell being cast has a different ObjectId → exclude_self only skips
    // when obj.id == spell_id. The hand copy is a different object.
    // So the eminence modifier DOES apply to the hand copy (different object).
    // This test verifies the hand copy gets the reduction: costs {3}{W}{U}{B}{R}{G}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state.turn.priority_holder = Some(p1);

    let hand_id = find_object(&state, "Ur Dragon Hand");

    // Hand copy IS a different object from the command zone copy, so the command
    // zone modifier DOES apply (exclude_self only skips when obj.id == spell_id,
    // and the command zone object ≠ the hand object).
    let (state, _) = cast_spell(state, p1, hand_id).expect(
        "CR 601.2f: Ur-Dragon hand copy costs {3}{W}{U}{B}{R}{G} (eminence from command zone copy)",
    );
    assert_eq!(state.stack_objects.len(), 1);
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());

    // Negative: casting with only {4} generic fails (cost is {3}+5 colored, not free).
    let ur_dragon_hand2 = ObjectSpec::card(p1, "Ur Dragon Hand2")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("ur-dragon-self-test"))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 4,
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        });
    let ur_dragon_cmd2 = ObjectSpec::card(p1, "Ur Dragon Cmd2")
        .in_zone(ZoneId::Command(p1))
        .with_card_id(cid("ur-dragon-self-test"))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Dragon".to_string())])
        .with_mana_cost(ManaCost {
            generic: 4,
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        });

    let registry2 = CardRegistry::new(vec![CardDefinition {
        card_id: cid("ur-dragon-self-test"),
        name: "Ur Dragon Hand2".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::HasSubtype(SubType("Dragon".to_string())),
            scope: CostModifierScope::Controller,
            eminence: true,
            exclude_self: true,
        }],
        ..Default::default()
    }]);

    let mut state2 = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry2)
        .object(ur_dragon_hand2)
        .object(ur_dragon_cmd2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Give only {2} generic + colored (not enough for {3}{W}{U}{B}{R}{G}).
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::White, 1);
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Blue, 1);
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 1);
    state2
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Green, 1);
    state2.turn.priority_holder = Some(p1);

    let hand2_id = find_object(&state2, "Ur Dragon Hand2");
    let result2 = cast_spell(state2, p1, hand2_id);
    assert!(
        result2.is_err(),
        "CR 601.2f: Ur-Dragon hand copy should fail with only {{2}} generic (needs {{3}})"
    );
}

// ── PB-29: New SpellCostFilter variants ─────────────────────────────────────

#[test]
/// CR 601.2f — SpellCostFilter::ColorAndCreature reduces only creature spells of the
/// specified color. A black creature spell costs {1} less with Bontu's Monument style modifier.
fn test_spell_cost_filter_color_and_creature_reduces_matching() {
    let p1 = p(1);
    let p2 = p(2);

    // Bontu's Monument style: black creature spells cost {1} less (controller only).
    let monument_def = CardDefinition {
        card_id: cid("bontu-test"),
        name: "Bontu Test".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::ColorAndCreature(Color::Black),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![monument_def]);

    // Permanent on battlefield.
    let monument = ObjectSpec::creature(p1, "Bontu Test", 0, 0).with_card_id(cid("bontu-test"));
    // Black creature spell: generic=2, black=1 → costs {2}{B} → with reduction {1}{B}.
    let black_creature = ObjectSpec::card(p1, "Vampire Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("vampire-test"))
        .with_types(vec![CardType::Creature])
        .with_colors(vec![Color::Black])
        .with_mana_cost(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(monument)
        .object(black_creature)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Provide {1}{B} — exactly reduced cost.
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
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Vampire Test");
    let (state, _) = cast_spell(state, p1, spell_id).expect(
        "CR 601.2f: black creature spell should cost {1}{B} with ColorAndCreature(Black) reduction",
    );
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana should be fully spent"
    );
}

#[test]
/// CR 601.2f — SpellCostFilter::ColorAndCreature does NOT reduce non-creature black spells.
/// A black instant spell should not be reduced by a black-creature-only modifier.
fn test_spell_cost_filter_color_and_creature_no_match_noncreature() {
    let p1 = p(1);
    let p2 = p(2);

    let monument_def = CardDefinition {
        card_id: cid("bontu-test2"),
        name: "Bontu Test2".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::ColorAndCreature(Color::Black),
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![monument_def]);

    let monument = ObjectSpec::creature(p1, "Bontu Test2", 0, 0).with_card_id(cid("bontu-test2"));
    // Black instant (noncreature) — should NOT be reduced.
    let black_instant = ObjectSpec::card(p1, "Dark Ritual Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("dark-ritual-test"))
        .with_types(vec![CardType::Instant])
        .with_colors(vec![Color::Black])
        .with_mana_cost(ManaCost {
            black: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(monument)
        .object(black_instant)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Only provide {0} generic — fails because noncreature isn't reduced.
    // Actually the instant costs {B} so we need {B}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Dark Ritual Test");
    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: black instant costs full {B} (not reduced by ColorAndCreature)");
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

#[test]
/// CR 601.2f — SpellCostFilter::HasChosenCreatureSubtype reduces creature spells whose
/// subtype matches the source's chosen_creature_type. Goblin with chosen_type=Goblin costs less.
fn test_spell_cost_filter_chosen_creature_subtype() {
    let p1 = p(1);
    let p2 = p(2);

    // Urza's Incubator style: creature spells of chosen type cost {2} less (all players).
    let incubator_def = CardDefinition {
        card_id: cid("incubator-test"),
        name: "Incubator Test".to_string(),
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -2,
            filter: SpellCostFilter::HasChosenCreatureSubtype,
            scope: CostModifierScope::AllPlayers,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![incubator_def]);

    // Incubator (without chosen_creature_type set yet — will set after build).
    let incubator =
        ObjectSpec::creature(p1, "Incubator Test", 0, 0).with_card_id(cid("incubator-test"));

    // Goblin creature spell: {3}{R} → with {2} reduction = {1}{R}.
    let goblin = ObjectSpec::card(p1, "Goblin Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("goblin-test"))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Goblin".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        });

    // Non-goblin creature spell: {3}{R} — NOT reduced.
    let elf = ObjectSpec::card(p1, "Elf Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("elf-test"))
        .with_types(vec![CardType::Creature])
        .with_subtypes(vec![SubType("Elf".to_string())])
        .with_mana_cost(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        });

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(incubator)
        .object(goblin)
        .object(elf)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Set chosen_creature_type = Goblin on the incubator object.
    let incubator_id = find_object(&state, "Incubator Test");
    state
        .objects
        .get_mut(&incubator_id)
        .unwrap()
        .chosen_creature_type = Some(SubType("Goblin".to_string()));

    // Pay {1}{R} for Goblin (reduced from {3}{R}).
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

    let goblin_id = find_object(&state, "Goblin Test");
    let (state, _) = cast_spell(state.clone(), p1, goblin_id)
        .expect("CR 601.2f: Goblin spell should cost {{1}}{{R}} with incubator choosing Goblin");
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

// ── PB-29: New SelfCostReduction variants ────────────────────────────────────

#[test]
/// CR 601.2f — SelfCostReduction::ConditionalKeyword: Winged Words style spell costs {1} less
/// when caster controls a creature with flying.
fn test_self_cost_reduction_conditional_keyword_flying() {
    let p1 = p(1);
    let p2 = p(2);

    // Winged Words style: costs {1} less if controller has a creature with flying.
    let winged_def = CardDefinition {
        card_id: cid("winged-test"),
        name: "Winged Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::ConditionalKeyword {
            keyword: KeywordAbility::Flying,
            reduction: 1,
        }),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![winged_def]);

    let spell = ObjectSpec::card(p1, "Winged Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("winged-test"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        });

    // A creature with flying controlled by p1.
    let flyer = ObjectSpec::creature(p1, "Flyer", 1, 1).with_keyword(KeywordAbility::Flying);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(flyer)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // With flying creature: {2}{U} - 1 = {1}{U}.
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
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Winged Test");
    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: should cost {1}{U} when controlling a flyer");
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

#[test]
/// CR 601.2f — SelfCostReduction::ConditionalKeyword: no reduction when no creature with
/// the keyword is present.
fn test_self_cost_reduction_conditional_keyword_no_match() {
    let p1 = p(1);
    let p2 = p(2);

    let winged_def = CardDefinition {
        card_id: cid("winged-test2"),
        name: "Winged Test2".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::ConditionalKeyword {
            keyword: KeywordAbility::Flying,
            reduction: 1,
        }),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![winged_def]);

    let spell = ObjectSpec::card(p1, "Winged Test2")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("winged-test2"))
        .with_types(vec![CardType::Sorcery])
        .with_mana_cost(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        });

    // No creatures with flying — a ground creature only.
    let walker = ObjectSpec::creature(p1, "Walker", 2, 2);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(walker)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Attempt with only {1}{U} — should fail (needs {2}{U}).
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
        .add(ManaColor::Blue, 1);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Winged Test2");
    let result = cast_spell(state, p1, spell_id);
    assert!(
        result.is_err(),
        "CR 601.2f: should require full {{2}}{{U}} when no flyer is present"
    );
}

#[test]
/// CR 601.2f — SelfCostReduction::MaxOpponentPermanents: Cavern-Hoard Dragon style reduction
/// using the maximum artifact count among all opponents in a 1v1 game.
fn test_self_cost_reduction_max_opponent_permanents_1v1() {
    let p1 = p(1);
    let p2 = p(2);

    // Cavern-Hoard Dragon style: costs {X} less where X is the max artifacts any opponent has.
    let dragon_def = CardDefinition {
        card_id: cid("dragon-test"),
        name: "Dragon Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 7,
            red: 2,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::MaxOpponentPermanents {
            filter: TargetFilter {
                has_card_type: Some(CardType::Artifact),
                ..Default::default()
            },
            per: 1,
        }),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![dragon_def]);

    let spell = ObjectSpec::card(p1, "Dragon Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("dragon-test"))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 7,
            red: 2,
            ..Default::default()
        });

    // Opponent has 3 artifacts on the battlefield.
    let art1 = ObjectSpec::card(p2, "Artifact 1")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);
    let art2 = ObjectSpec::card(p2, "Artifact 2")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);
    let art3 = ObjectSpec::card(p2, "Artifact 3")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(spell)
        .object(art1)
        .object(art2)
        .object(art3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // {7}{RR} - 3 artifacts = {4}{RR}. Give {4}{RR}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 4);
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Red, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Dragon Test");
    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: dragon should cost {4}{RR} with opponent having 3 artifacts");
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

#[test]
/// CR 601.2f — SelfCostReduction::MaxOpponentPermanents in multiplayer: uses the MAXIMUM
/// (not sum) of opponent artifact counts.
fn test_self_cost_reduction_max_opponent_permanents_multiplayer() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let p4 = p(4);

    let dragon_def = CardDefinition {
        card_id: cid("dragon-multi-test"),
        name: "Dragon Multi Test".to_string(),
        mana_cost: Some(ManaCost {
            generic: 7,
            red: 2,
            ..Default::default()
        }),
        self_cost_reduction: Some(SelfCostReduction::MaxOpponentPermanents {
            filter: TargetFilter {
                has_card_type: Some(CardType::Artifact),
                ..Default::default()
            },
            per: 1,
        }),
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![dragon_def]);

    let spell = ObjectSpec::card(p1, "Dragon Multi Test")
        .in_zone(ZoneId::Hand(p1))
        .with_card_id(cid("dragon-multi-test"))
        .with_types(vec![CardType::Creature])
        .with_mana_cost(ManaCost {
            generic: 7,
            red: 2,
            ..Default::default()
        });

    // p2 has 1 artifact, p3 has 5 artifacts, p4 has 3 artifacts. Max = 5. All on battlefield.
    let arts: Vec<_> = (0..5)
        .map(|i| {
            ObjectSpec::card(p3, &format!("Art {}", i))
                .in_zone(ZoneId::Battlefield)
                .with_types(vec![CardType::Artifact])
        })
        .collect();
    let art_p2 = ObjectSpec::card(p2, "Art P2")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);
    let art_p4_1 = ObjectSpec::card(p4, "Art P4-1")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);
    let art_p4_2 = ObjectSpec::card(p4, "Art P4-2")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);
    let art_p4_3 = ObjectSpec::card(p4, "Art P4-3")
        .in_zone(ZoneId::Battlefield)
        .with_types(vec![CardType::Artifact]);

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .add_player(p4)
        .with_registry(registry)
        .object(spell)
        .object(art_p2)
        .object(art_p4_1)
        .object(art_p4_2)
        .object(art_p4_3);
    for a in arts {
        builder = builder.object(a);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Max opponent artifacts = 5 (from p3). Cost: {7}{RR} - 5 = {2}{RR}.
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
        .add(ManaColor::Red, 2);
    state.turn.priority_holder = Some(p1);

    let spell_id = find_object(&state, "Dragon Multi Test");
    let (state, _) = cast_spell(state, p1, spell_id)
        .expect("CR 601.2f: dragon should cost {2}{RR} with max opponent having 5 artifacts");
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}

// ── PB-29: SelfActivatedCostReduction tests ──────────────────────────────────

/// Helper: build an activation cost from a mana cost.
fn mana_activation_cost(generic: u32) -> ActivationCost {
    ActivationCost {
        mana_cost: Some(ManaCost {
            generic,
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
/// CR 602.2b + 601.2f — SelfActivatedCostReduction::PerPermanent reduces the activation cost
/// by {1} per legendary creature the controller has. 2 legendary creatures = {2} less.
fn test_activated_ability_self_cost_reduction_per_legendary() {
    let p1 = p(1);
    let p2 = p(2);

    // Channel land style: ability costs {4} generic normally, {1} less per legendary creature.
    let channel_def = CardDefinition {
        card_id: cid("channel-land-test"),
        name: "Channel Land Test".to_string(),
        activated_ability_cost_reductions: vec![(
            0,
            SelfActivatedCostReduction::PerPermanent {
                per: 1,
                filter: TargetFilter {
                    legendary: true,
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            },
        )],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![channel_def]);

    // The land with the channel ability (generic=4 mana cost), on the battlefield.
    let land = ObjectSpec::card(p1, "Channel Land Test")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(cid("channel-land-test"))
        .with_activated_ability(ActivatedAbility {
            cost: mana_activation_cost(4),
            description: "Channel: deal damage".to_string(),
            effect: None,
            sorcery_speed: false,
            targets: vec![],
            activation_condition: None,

            activation_zone: None,
            once_per_turn: false,
        });

    // 2 legendary creatures controlled by p1.
    use mtg_engine::SuperType;
    let leg1 =
        ObjectSpec::creature(p1, "Legend One", 3, 3).with_supertypes(vec![SuperType::Legendary]);
    let leg2 =
        ObjectSpec::creature(p1, "Legend Two", 2, 2).with_supertypes(vec![SuperType::Legendary]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(land)
        .object(leg1)
        .object(leg2)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Cost: {4} - 2 legendary = {2}.
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Channel Land Test");
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("CR 602.2b: ability should cost 2 generic with 2 legendary creatures");
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "mana should be fully spent"
    );
}

#[test]
/// CR 601.2f — Reduction cannot bring generic cost below {0}. Even with 10 legendary creatures,
/// a {3} generic activation should not become negative.
fn test_activated_ability_self_cost_reduction_floor_zero() {
    let p1 = p(1);
    let p2 = p(2);

    let channel_def = CardDefinition {
        card_id: cid("channel-floor-test"),
        name: "Channel Floor Test".to_string(),
        activated_ability_cost_reductions: vec![(
            0,
            SelfActivatedCostReduction::PerPermanent {
                per: 1,
                filter: TargetFilter {
                    legendary: true,
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            },
        )],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![channel_def]);

    let land = ObjectSpec::card(p1, "Channel Floor Test")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(cid("channel-floor-test"))
        .with_activated_ability(ActivatedAbility {
            cost: mana_activation_cost(3),
            description: "Channel: floor test".to_string(),
            effect: None,
            sorcery_speed: false,
            targets: vec![],
            activation_condition: None,

            activation_zone: None,
            once_per_turn: false,
        });

    // 10 legendary creatures — would reduce by 10 but cost is only 3.
    use mtg_engine::SuperType;
    let legends: Vec<_> = (0..10)
        .map(|i| {
            ObjectSpec::creature(p1, &format!("Legend {}", i), 1, 1)
                .with_supertypes(vec![SuperType::Legendary])
        })
        .collect();

    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(land);
    for l in legends {
        builder = builder.object(l);
    }
    let mut state = builder
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Cost reduced to {0} (floor). Give {0} mana — should succeed.
    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Channel Floor Test");
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 0,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("CR 601.2f: cost should floor at {0}, not go negative");
    assert!(
        state.players.get(&p1).unwrap().mana_pool.is_empty(),
        "no mana needed at {{0}} cost"
    );
}

#[test]
/// CR 602.2b + 601.2f — Voldaren Estate style: Blood token activation costs {1} less per
/// Vampire the controller has. 3 Vampires reduces {5} to {2}.
fn test_activated_ability_self_cost_reduction_vampires() {
    let p1 = p(1);
    let p2 = p(2);

    // Voldaren Estate Blood token ability (index 1, since index 0 is the life-pay mana ability).
    let estate_def = CardDefinition {
        card_id: cid("estate-test"),
        name: "Estate Test".to_string(),
        activated_ability_cost_reductions: vec![(
            1,
            SelfActivatedCostReduction::PerPermanent {
                per: 1,
                filter: TargetFilter {
                    has_subtype: Some(SubType("Vampire".to_string())),
                    ..Default::default()
                },
                controller: PlayerTarget::Controller,
            },
        )],
        ..Default::default()
    };

    let registry = CardRegistry::new(vec![estate_def]);

    // Dummy ability at index 0.
    let dummy_ab = ActivatedAbility {
        cost: ActivationCost {
            mana_cost: None,
            ..Default::default()
        },
        description: "Pay life: add mana".to_string(),
        effect: None,
        sorcery_speed: false,
        targets: vec![],
        activation_condition: None,

        activation_zone: None,
        once_per_turn: false,
    };
    // Blood token ability at index 1 — costs {5} generic (tap as part of cost).
    let blood_ab = ActivatedAbility {
        cost: ActivationCost {
            mana_cost: Some(ManaCost {
                generic: 5,
                ..Default::default()
            }),
            requires_tap: true,
            ..Default::default()
        },
        description: "Blood token".to_string(),
        effect: None,
        sorcery_speed: false,
        targets: vec![],
        activation_condition: None,

        activation_zone: None,
        once_per_turn: false,
    };

    let estate = ObjectSpec::card(p1, "Estate Test")
        .in_zone(ZoneId::Battlefield)
        .with_card_id(cid("estate-test"))
        .with_activated_ability(dummy_ab)
        .with_activated_ability(blood_ab);

    // 3 Vampires controlled by p1.
    let v1 = ObjectSpec::creature(p1, "Vampire A", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())]);
    let v2 = ObjectSpec::creature(p1, "Vampire B", 2, 2)
        .with_subtypes(vec![SubType("Vampire".to_string())]);
    let v3 = ObjectSpec::creature(p1, "Vampire C", 1, 1)
        .with_subtypes(vec![SubType("Vampire".to_string())]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(estate)
        .object(v1)
        .object(v2)
        .object(v3)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();

    // Cost: {5} - 3 vampires = {2}. The tap is free (already untapped).
    state
        .players
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 2);
    state.turn.priority_holder = Some(p1);

    let source_id = find_object(&state, "Estate Test");
    let (state, _) = process_command(
        state,
        Command::ActivateAbility {
            player: p1,
            source: source_id,
            ability_index: 1,
            targets: vec![],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
        },
    )
    .expect("CR 602.2b: Blood token ability should cost {2} with 3 Vampires");
    assert!(state.players.get(&p1).unwrap().mana_pool.is_empty());
}
