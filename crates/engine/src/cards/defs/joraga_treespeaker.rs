// Joraga Treespeaker — {G}, Creature — Elf Druid 1/1
// Level up {1}{G}
// LEVEL 1-4: 1/2, {T}: Add {G}{G}.
// LEVEL 5+: 1/4, Elves you control have "{T}: Add {G}{G}."
//
// TODO: Level up mechanic not in DSL — no LevelUp keyword or level-based ability gating.
// Implementing only the base body and the level up activated ability (adds level counters).
// The level-dependent abilities and P/T changes are not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("joraga-treespeaker"),
        name: "Joraga Treespeaker".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "Level up {1}{G} ({1}{G}: Put a level counter on this. Level up only as a sorcery.)\nLEVEL 1-4\n1/2\n{T}: Add {G}{G}.\nLEVEL 5+\n1/4\nElves you control have \"{T}: Add {G}{G}.\"".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Level up {1}{G}: put a level counter on this (sorcery speed only)
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, green: 1, ..Default::default() }),
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Level,
                    count: 1,
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // TODO: Level-dependent abilities and P/T changes (Level 1-4: 1/2 + {T}: Add {G}{G};
            //       Level 5+: 1/4 + grant Elves "{T}: Add {G}{G}") not expressible in DSL.
        ],
        ..Default::default()
    }
}
