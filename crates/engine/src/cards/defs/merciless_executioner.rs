// Merciless Executioner — {2}{B}, Creature — Orc Warrior 3/1
// When this enters, each player sacrifices a creature.
//
// Note: Functionally identical to Fleshbag Marauder. Different creature types (Orc Warrior vs
// Zombie Warrior). SacrificePermanents has no creature-only filter — picks lowest-ID permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("merciless-executioner"),
        name: "Merciless Executioner".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Orc", "Warrior"]),
        oracle_text: "When this enters, each player sacrifices a creature.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            // CR 603.3: ETB trigger — each player sacrifices a creature.
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
