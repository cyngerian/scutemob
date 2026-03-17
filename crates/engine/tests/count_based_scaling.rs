//! Tests for EffectAmount::PermanentCount, DevotionTo, and CounterCount (PB-7).
//!
//! CR 700.5: Devotion to [color] = count of mana symbols of that color in mana
//! costs of permanents you control.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::{
    CardEffectTarget, CardType, Color, CounterType, Effect, EffectAmount, GameStateBuilder,
    HybridMana, ManaCost, ManaColor, ObjectId, ObjectSpec, PhyrexianMana, PlayerId, PlayerTarget,
    TargetFilter, ZoneId,
};

fn p1() -> PlayerId {
    PlayerId(1)
}
fn p2() -> PlayerId {
    PlayerId(2)
}

fn creature_filter() -> TargetFilter {
    TargetFilter {
        has_card_type: Some(CardType::Creature),
        ..Default::default()
    }
}

fn find_on_battlefield(state: &mtg_engine::GameState, name: &str) -> ObjectId {
    let bf = state.zones.get(&ZoneId::Battlefield).unwrap();
    *bf.object_ids()
        .iter()
        .find(|id| {
            state
                .objects
                .get(id)
                .map(|o| o.characteristics.name == name)
                .unwrap_or(false)
        })
        .unwrap_or_else(|| panic!("object '{}' not found on battlefield", name))
}

// ---------------------------------------------------------------------------
// PermanentCount tests
// ---------------------------------------------------------------------------

/// PermanentCount counts creatures controlled by the effect's controller.
#[test]
fn test_permanent_count_creatures_you_control() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Bear A", 2, 2))
        .object(ObjectSpec::creature(p1(), "Bear B", 2, 2))
        .object(ObjectSpec::creature(p1(), "Bear C", 2, 2))
        .object(ObjectSpec::creature(p2(), "Enemy Bear", 2, 2))
        .build()
        .unwrap();

    let source = find_on_battlefield(&state, "Bear A");

    // P1 controls 3 creatures, so should gain 3 life.
    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::PermanentCount {
            filter: creature_filter(),
            controller: PlayerTarget::Controller,
        },
    };

    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(state.players.get(&p1()).unwrap().life_total, 43);
}

/// PermanentCount with EachOpponent counts opponents' permanents.
#[test]
fn test_permanent_count_opponents_creatures() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "P1 Bear", 2, 2))
        .object(ObjectSpec::creature(p2(), "P2 Bear A", 2, 2))
        .object(ObjectSpec::creature(p2(), "P2 Bear B", 2, 2))
        .build()
        .unwrap();

    let source = find_on_battlefield(&state, "P1 Bear");

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::PermanentCount {
            filter: creature_filter(),
            controller: PlayerTarget::EachOpponent,
        },
    };

    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    // P2 controls 2 creatures
    assert_eq!(state.players.get(&p1()).unwrap().life_total, 42);
}

/// PermanentCount with a land filter counts lands, not creatures.
#[test]
fn test_permanent_count_lands_you_control() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Bear", 2, 2))
        .object(
            ObjectSpec::card(p1(), "Forest A")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p1(), "Forest B")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::card(p2(), "Enemy Forest")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .build()
        .unwrap();

    let source = find_on_battlefield(&state, "Bear");

    let land_filter = TargetFilter {
        has_card_type: Some(CardType::Land),
        ..Default::default()
    };

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::PermanentCount {
            filter: land_filter,
            controller: PlayerTarget::Controller,
        },
    };

    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    // P1 controls 2 lands
    assert_eq!(state.players.get(&p1()).unwrap().life_total, 42);
}

// ---------------------------------------------------------------------------
// DevotionTo tests
// ---------------------------------------------------------------------------

/// CR 700.5: Devotion to green = count of {G} symbols in mana costs of permanents
/// you control.
#[test]
fn test_devotion_to_color_counts_mana_symbols() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        // {G} — 1 green symbol
        .object(
            ObjectSpec::creature(p1(), "Llanowar Elves", 1, 1).with_mana_cost(ManaCost {
                green: 1,
                ..Default::default()
            }),
        )
        // {1}{G} — 1 green symbol
        .object(
            ObjectSpec::creature(p1(), "Grizzly Bears", 2, 2).with_mana_cost(ManaCost {
                generic: 1,
                green: 1,
                ..Default::default()
            }),
        )
        // {G}{G}{G} — 3 green symbols
        .object(
            ObjectSpec::creature(p1(), "Leatherback Baloth", 4, 5).with_mana_cost(ManaCost {
                green: 3,
                ..Default::default()
            }),
        )
        // Opponent's creature — doesn't count
        .object(
            ObjectSpec::creature(p2(), "Enemy Elf", 1, 1).with_mana_cost(ManaCost {
                green: 2,
                ..Default::default()
            }),
        )
        .build()
        .unwrap();

    let source = find_on_battlefield(&state, "Llanowar Elves");

    // Devotion to green: 1 + 1 + 3 = 5
    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::DevotionTo(Color::Green),
    };

    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(state.players.get(&p1()).unwrap().life_total, 45);
}

/// Devotion to a color with no matching mana symbols yields 0.
#[test]
fn test_devotion_to_color_zero_when_no_symbols() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::creature(p1(), "Grizzly Bears", 2, 2).with_mana_cost(ManaCost {
                generic: 1,
                green: 1,
                ..Default::default()
            }),
        )
        .build()
        .unwrap();

    let source = find_on_battlefield(&state, "Grizzly Bears");

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::DevotionTo(Color::Blue),
    };

    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(state.players.get(&p1()).unwrap().life_total, 40);
}

/// Devotion excludes permanents without mana costs (e.g., basic lands).
#[test]
fn test_devotion_excludes_permanents_without_mana_cost() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(
            ObjectSpec::card(p1(), "Forest")
                .with_types(vec![CardType::Land])
                .in_zone(ZoneId::Battlefield),
        )
        .object(
            ObjectSpec::creature(p1(), "Elf", 1, 1).with_mana_cost(ManaCost {
                green: 1,
                ..Default::default()
            }),
        )
        .build()
        .unwrap();

    let source = find_on_battlefield(&state, "Elf");

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::DevotionTo(Color::Green),
    };

    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    // Only the Elf contributes: devotion = 1
    assert_eq!(state.players.get(&p1()).unwrap().life_total, 41);
}

/// CR 700.5: Hybrid and phyrexian mana symbols count toward devotion.
/// A permanent with {G/W}{G/W} contributes 2 to both green and white devotion.
/// A permanent with {G/P} (phyrexian green) contributes 1 to green devotion.
#[test]
fn test_devotion_counts_hybrid_and_phyrexian_mana_symbols() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        // {G/W}{G/W} — 2 hybrid symbols, each counting for both green and white
        .object(
            ObjectSpec::creature(p1(), "Hybrid Druid", 2, 2).with_mana_cost(ManaCost {
                hybrid: vec![
                    HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
                    HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
                ],
                ..Default::default()
            }),
        )
        // {2/G} — GenericColor hybrid, counts 1 toward green
        .object(
            ObjectSpec::creature(p1(), "Mono Hybrid Elf", 1, 1).with_mana_cost(ManaCost {
                hybrid: vec![HybridMana::GenericColor(ManaColor::Green)],
                ..Default::default()
            }),
        )
        // {G/P} — phyrexian green, counts 1 toward green
        .object(
            ObjectSpec::creature(p1(), "Phyrexian Elf", 1, 1).with_mana_cost(ManaCost {
                phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
                ..Default::default()
            }),
        )
        .build()
        .unwrap();

    let source = find_on_battlefield(&state, "Hybrid Druid");

    // Green devotion: 2 (from {G/W}{G/W}) + 1 (from {2/G}) + 1 (from {G/P}) = 4
    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::DevotionTo(Color::Green),
    };
    let mut ctx = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);
    assert_eq!(state.players.get(&p1()).unwrap().life_total, 44);

    // White devotion: 2 (from {G/W}{G/W}), none from the others
    let effect_white = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::DevotionTo(Color::White),
    };
    let mut ctx2 = EffectContext::new(p1(), source, vec![]);
    execute_effect(&mut state, &effect_white, &mut ctx2);
    assert_eq!(state.players.get(&p1()).unwrap().life_total, 46);
}

// ---------------------------------------------------------------------------
// CounterCount tests
// ---------------------------------------------------------------------------

/// CounterCount counts +1/+1 counters on a specific permanent.
#[test]
fn test_counter_count_on_self() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Hydra", 0, 0))
        .build()
        .unwrap();

    let hydra = find_on_battlefield(&state, "Hydra");

    // Place 5 +1/+1 counters on the Hydra
    {
        let obj = state.objects.get_mut(&hydra).unwrap();
        obj.counters.insert(CounterType::PlusOnePlusOne, 5);
    }

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::CounterCount {
            target: CardEffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
        },
    };

    let mut ctx = EffectContext::new(p1(), hydra, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(state.players.get(&p1()).unwrap().life_total, 45);
}

/// CounterCount returns 0 when the permanent has no counters of that type.
#[test]
fn test_counter_count_zero_when_no_counters() {
    let mut state = GameStateBuilder::new()
        .add_player(p1())
        .add_player(p2())
        .object(ObjectSpec::creature(p1(), "Bear", 2, 2))
        .build()
        .unwrap();

    let bear = find_on_battlefield(&state, "Bear");

    let effect = Effect::GainLife {
        player: PlayerTarget::Controller,
        amount: EffectAmount::CounterCount {
            target: CardEffectTarget::Source,
            counter: CounterType::PlusOnePlusOne,
        },
    };

    let mut ctx = EffectContext::new(p1(), bear, vec![]);
    execute_effect(&mut state, &effect, &mut ctx);

    assert_eq!(state.players.get(&p1()).unwrap().life_total, 40);
}
