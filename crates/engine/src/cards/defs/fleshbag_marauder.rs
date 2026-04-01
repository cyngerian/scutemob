// Fleshbag Marauder — {2}{B}, Creature — Zombie Warrior 3/1
// When this enters, each player sacrifices a creature.
//
// Note: SacrificePermanents has no creature-only filter; engine picks lowest-ID permanent.
// "Non-token" and "creature specifically" filters are a known DSL gap (same as Butcher of Malakir).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fleshbag-marauder"),
        name: "Fleshbag Marauder".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Zombie", "Warrior"]),
        oracle_text: "When this enters, each player sacrifices a creature.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            // CR 603.3: ETB trigger — each player sacrifices a creature.
            // SacrificePermanents with EachPlayer fires for all players simultaneously.
            // TODO: SacrificePermanents lacks creature-only filter — picks any permanent.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
