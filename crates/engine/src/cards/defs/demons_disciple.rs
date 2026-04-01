// Demon's Disciple — {2}{B}, Creature — Human Cleric 3/1
// When this enters, each player sacrifices a creature.
// TODO: SacrificePermanents has no creature-only filter — each player sacrifices any
// permanent, not specifically a creature. Stronger than intended for permanents-matter boards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("demons-disciple"),
        name: "Demon's Disciple".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "When this enters, each player sacrifices a creature.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                // TODO: no creature-only filter on SacrificePermanents; sacrifices any permanent.
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
