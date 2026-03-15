//! Tests for mana spending restrictions (CR 106.12) and ETB creature type choice.
//!
//! PB-11: ManaRestriction enum, restricted mana pool tracking,
//! chosen_creature_type on GameObject, ChooseCreatureType effect.

use mtg_engine::cards::card_definition::{
    AbilityDefinition, CardDefinition, Cost, Effect, ManaRestriction, PlayerTarget, TypeLine,
};
use mtg_engine::rules::{process_command, Command};
use mtg_engine::state::game_object::ManaCost;
use mtg_engine::state::player::SpellContext;
use mtg_engine::state::replacement_effect::{ReplacementModification, ReplacementTrigger};
use mtg_engine::state::turn::Step;
use mtg_engine::state::{
    CardId, CardType, GameStateBuilder, ManaPool, ObjectSpec, PlayerId, SubType, ZoneId,
};
use mtg_engine::CardRegistry;

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn st(s: &str) -> SubType {
    SubType(s.to_string())
}

/// CR 106.12 — restricted mana of matching color can be spent on matching spell
#[test]
fn test_mana_restriction_pool_restricted_available_creature_only() {
    let mut pool = ManaPool::default();
    pool.add_restricted(
        mtg_engine::ManaColor::Green,
        2,
        ManaRestriction::CreatureSpellsOnly,
    );
    pool.add(mtg_engine::ManaColor::Green, 1);

    let creature_spell = SpellContext {
        is_creature: true,
        subtypes: vec![],
    };
    let non_creature_spell = SpellContext {
        is_creature: false,
        subtypes: vec![],
    };

    // Restricted mana available for creature spell
    assert_eq!(
        pool.restricted_available(mtg_engine::ManaColor::Green, &creature_spell),
        2
    );
    // Not available for non-creature spell
    assert_eq!(
        pool.restricted_available(mtg_engine::ManaColor::Green, &non_creature_spell),
        0
    );
    // Unrestricted mana is separate
    assert_eq!(pool.green, 1);
    // Total with restricted
    assert_eq!(pool.total_with_restricted(), 3);
}

/// CR 106.12 — SubtypeOnly restriction only matches spells of that subtype
#[test]
fn test_mana_restriction_subtype_only_dragon() {
    let mut pool = ManaPool::default();
    pool.add_restricted(
        mtg_engine::ManaColor::Red,
        1,
        ManaRestriction::SubtypeOnly(st("Dragon")),
    );

    let dragon_spell = SpellContext {
        is_creature: true,
        subtypes: vec![st("Dragon")],
    };
    let elf_spell = SpellContext {
        is_creature: true,
        subtypes: vec![st("Elf")],
    };

    assert_eq!(
        pool.restricted_available(mtg_engine::ManaColor::Red, &dragon_spell),
        1
    );
    assert_eq!(
        pool.restricted_available(mtg_engine::ManaColor::Red, &elf_spell),
        0
    );
}

/// CR 106.12 — SubtypeOrSubtype restriction matches either subtype
#[test]
fn test_mana_restriction_subtype_or_subtype() {
    let mut pool = ManaPool::default();
    pool.add_restricted(
        mtg_engine::ManaColor::Blue,
        1,
        ManaRestriction::SubtypeOrSubtype(st("Dragon"), st("Omen")),
    );

    let dragon_spell = SpellContext {
        is_creature: true,
        subtypes: vec![st("Dragon")],
    };
    let omen_spell = SpellContext {
        is_creature: false,
        subtypes: vec![st("Omen")],
    };
    let elf_spell = SpellContext {
        is_creature: true,
        subtypes: vec![st("Elf")],
    };

    assert_eq!(
        pool.restricted_available(mtg_engine::ManaColor::Blue, &dragon_spell),
        1
    );
    assert_eq!(
        pool.restricted_available(mtg_engine::ManaColor::Blue, &omen_spell),
        1
    );
    assert_eq!(
        pool.restricted_available(mtg_engine::ManaColor::Blue, &elf_spell),
        0
    );
}

/// CR 106.12 — spend_restricted deducts from matching restricted entries
#[test]
fn test_mana_restriction_spend_restricted() {
    let mut pool = ManaPool::default();
    pool.add_restricted(
        mtg_engine::ManaColor::Green,
        3,
        ManaRestriction::CreatureSpellsOnly,
    );

    let creature_spell = SpellContext {
        is_creature: true,
        subtypes: vec![],
    };

    let spent = pool.spend_restricted(mtg_engine::ManaColor::Green, 2, &creature_spell);
    assert_eq!(spent, 2);
    // 1 remaining restricted
    assert_eq!(
        pool.restricted_available(mtg_engine::ManaColor::Green, &creature_spell),
        1
    );
}

/// CR 106.12 — restricted mana merges with existing entry of same color/restriction
#[test]
fn test_mana_restriction_add_restricted_merges() {
    let mut pool = ManaPool::default();
    pool.add_restricted(
        mtg_engine::ManaColor::White,
        1,
        ManaRestriction::CreatureSpellsOnly,
    );
    pool.add_restricted(
        mtg_engine::ManaColor::White,
        2,
        ManaRestriction::CreatureSpellsOnly,
    );

    // Should merge into one entry with amount 3
    assert_eq!(pool.restricted.len(), 1);
    assert_eq!(pool.restricted[0].amount, 3);
}

/// CR 106.12 — empty() clears restricted mana
#[test]
fn test_mana_restriction_empty_clears_restricted() {
    let mut pool = ManaPool::default();
    pool.add_restricted(
        mtg_engine::ManaColor::Red,
        5,
        ManaRestriction::SubtypeOnly(st("Elf")),
    );
    pool.add(mtg_engine::ManaColor::Red, 3);

    pool.empty();
    assert!(pool.is_empty());
    assert!(pool.restricted.is_empty());
    assert_eq!(pool.total_with_restricted(), 0);
}

/// CR 106.12 — ChooseCreatureType effect sets chosen_creature_type on permanent
#[test]
fn test_choose_creature_type_effect_sets_type() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![
        CardDefinition {
            name: "Type Chooser Land".to_string(),
            card_id: CardId("tcl".to_string()),
            types: TypeLine {
                card_types: [CardType::Land].into_iter().collect(),
                ..Default::default()
            },
            abilities: vec![
                // "As this enters, choose a creature type" — self-replacement effect
                AbilityDefinition::Replacement {
                    trigger: ReplacementTrigger::WouldEnterBattlefield {
                        filter: mtg_engine::state::replacement_effect::ObjectFilter::Any,
                    },
                    modification: ReplacementModification::ChooseCreatureType(st("Human")),
                    is_self: true,
                    unless_condition: None,
                },
            ],
            ..Default::default()
        },
        // A creature on the battlefield so the auto-pick has something to choose
        CardDefinition {
            name: "Test Elf".to_string(),
            card_id: CardId("telf".to_string()),
            types: TypeLine {
                card_types: [CardType::Creature].into_iter().collect(),
                subtypes: [st("Elf")].into_iter().collect(),
                ..Default::default()
            },
            power: Some(1),
            toughness: Some(1),
            ..Default::default()
        },
    ]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object(
            ObjectSpec::creature(p1, "Test Elf", 1, 1)
                .with_subtypes(vec![st("Elf")])
                .in_zone(ZoneId::Battlefield),
        )
        .object({
            let mut spec = ObjectSpec::card(p1, "Type Chooser Land")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Hand(p1));
            spec.card_id = Some(CardId("tcl".to_string()));
            spec
        })
        .build()
        .unwrap();

    // Set active player and step
    state.turn.active_player = p1;
    state.turn.step = Step::PreCombatMain;

    // Play the land
    let land_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Type Chooser Land")
        .unwrap()
        .id;
    let (state, _events) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: land_id,
        },
    )
    .unwrap();

    // Find the land on the battlefield (new ObjectId after zone change)
    let land = state
        .objects
        .values()
        .find(|o| {
            o.characteristics.name == "Type Chooser Land" && matches!(o.zone, ZoneId::Battlefield)
        })
        .expect("land should be on battlefield");

    // Should have chosen "Elf" since there's an Elf on the battlefield
    assert_eq!(
        land.chosen_creature_type,
        Some(st("Elf")),
        "chosen_creature_type should be Elf (most common creature type on battlefield)"
    );
}

/// CR 106.12 — ChooseCreatureType falls back to default when no creatures controlled
#[test]
fn test_choose_creature_type_fallback_default() {
    let p1 = p(1);
    let p2 = p(2);

    let registry = CardRegistry::new(vec![CardDefinition {
        name: "Type Chooser Land".to_string(),
        card_id: CardId("tcl".to_string()),
        types: TypeLine {
            card_types: [CardType::Land].into_iter().collect(),
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Replacement {
            trigger: ReplacementTrigger::WouldEnterBattlefield {
                filter: mtg_engine::state::replacement_effect::ObjectFilter::Any,
            },
            modification: ReplacementModification::ChooseCreatureType(st("Human")),
            is_self: true,
            unless_condition: None,
        }],
        ..Default::default()
    }]);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p(3))
        .add_player(p(4))
        .with_registry(registry)
        .object({
            let mut spec = ObjectSpec::card(p1, "Type Chooser Land")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Hand(p1));
            spec.card_id = Some(CardId("tcl".to_string()));
            spec
        })
        .build()
        .unwrap();

    state.turn.active_player = p1;
    state.turn.step = Step::PreCombatMain;

    let land_id = state
        .objects
        .values()
        .find(|o| o.characteristics.name == "Type Chooser Land")
        .unwrap()
        .id;
    let (state, _) = process_command(
        state,
        Command::PlayLand {
            player: p1,
            card: land_id,
        },
    )
    .unwrap();

    let land = state
        .objects
        .values()
        .find(|o| {
            o.characteristics.name == "Type Chooser Land" && matches!(o.zone, ZoneId::Battlefield)
        })
        .expect("land should be on battlefield");

    assert_eq!(
        land.chosen_creature_type,
        Some(st("Human")),
        "chosen_creature_type should fall back to default 'Human' when no creatures controlled"
    );
}

/// CR 106.12 — AddManaRestricted effect produces restricted mana in pool
#[test]
fn test_add_mana_restricted_effect_via_execute() {
    use mtg_engine::effects::{execute_effect, EffectContext};

    let p1 = p(1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .add_player(p(3))
        .add_player(p(4))
        .build()
        .unwrap();

    // Create a dummy source object for the effect context
    let source_id = state
        .objects
        .values()
        .next()
        .map(|o| o.id)
        .unwrap_or(mtg_engine::ObjectId(1));

    let mut ctx = EffectContext::new(p1, source_id, vec![]);

    let effect = Effect::AddManaRestricted {
        player: PlayerTarget::Controller,
        mana: ManaPool {
            green: 1,
            ..Default::default()
        },
        restriction: ManaRestriction::SubtypeOnly(st("Elf")),
    };

    let _events = execute_effect(&mut state, &effect, &mut ctx);

    let player = state.player(p1).unwrap();
    // Unrestricted mana should be 0
    assert_eq!(player.mana_pool.green, 0, "unrestricted green should be 0");
    // Restricted mana should have 1 green with Elf restriction
    assert_eq!(player.mana_pool.restricted.len(), 1);
    assert_eq!(
        player.mana_pool.restricted[0].color,
        mtg_engine::ManaColor::Green
    );
    assert_eq!(player.mana_pool.restricted[0].amount, 1);
    assert_eq!(
        player.mana_pool.restricted[0].restriction,
        ManaRestriction::SubtypeOnly(st("Elf"))
    );
}

/// CR 106.12 — can_pay_cost_with_context includes matching restricted mana
#[test]
fn test_can_pay_cost_with_matching_restricted_mana() {
    use mtg_engine::rules::casting::can_pay_cost_with_context;

    let mut pool = ManaPool::default();
    pool.add_restricted(
        mtg_engine::ManaColor::Green,
        1,
        ManaRestriction::SubtypeOnly(st("Elf")),
    );

    let cost = ManaCost {
        green: 1,
        ..Default::default()
    };

    let elf_spell = SpellContext {
        is_creature: true,
        subtypes: vec![st("Elf")],
    };
    let goblin_spell = SpellContext {
        is_creature: true,
        subtypes: vec![st("Goblin")],
    };

    // Can pay with restricted mana for Elf spell
    assert!(can_pay_cost_with_context(&pool, &cost, Some(&elf_spell)));
    // Cannot pay for Goblin spell (restricted mana doesn't match)
    assert!(!can_pay_cost_with_context(
        &pool,
        &cost,
        Some(&goblin_spell)
    ));
    // Cannot pay without context (restricted mana not counted)
    assert!(!can_pay_cost_with_context(&pool, &cost, None));
}

/// CR 106.12 — pay_cost_with_context spends restricted mana first
#[test]
fn test_pay_cost_with_context_spends_restricted_first() {
    use mtg_engine::rules::casting::pay_cost_with_context;

    let mut pool = ManaPool::default();
    pool.add(mtg_engine::ManaColor::Green, 2);
    pool.add_restricted(
        mtg_engine::ManaColor::Green,
        1,
        ManaRestriction::CreatureSpellsOnly,
    );

    let cost = ManaCost {
        green: 2,
        ..Default::default()
    };

    let creature_spell = SpellContext {
        is_creature: true,
        subtypes: vec![],
    };

    pay_cost_with_context(&mut pool, &cost, Some(&creature_spell));

    // Restricted mana should have been spent first (1 restricted + 1 unrestricted)
    assert_eq!(pool.restricted.len(), 0, "restricted should be fully spent");
    assert_eq!(pool.green, 1, "should have 1 unrestricted green remaining");
}
