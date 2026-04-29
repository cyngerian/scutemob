// Demon's Disciple — {2}{B}, Creature — Human Cleric 3/1
// When this enters, each player sacrifices a creature or planeswalker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("demons-disciple"),
        name: "Demon's Disciple".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "When this creature enters, each player sacrifices a creature or planeswalker of their choice.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            // CR 603.3: ETB trigger — each player sacrifices a creature or planeswalker.
            // PB-SFT (CR 701.17a + CR 109.1c): creature-or-planeswalker OR-type filter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                        ..Default::default()
                    }),
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
