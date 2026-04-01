// Accursed Marauder — {1}{B}, Creature — Zombie 2/1
// When this enters, each player sacrifices a nontoken creature.
// TODO: SacrificePermanents has no nontoken filter — each player sacrifices any permanent,
// not specifically a nontoken creature. Stronger than intended for token-heavy boards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("accursed-marauder"),
        name: "Accursed Marauder".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text: "When this enters, each player sacrifices a nontoken creature.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // When this enters, each player sacrifices a nontoken creature.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                // TODO: no nontoken filter available; sacrifices any permanent.
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
