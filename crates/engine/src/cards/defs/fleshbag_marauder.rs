// Fleshbag Marauder — {2}{B}, Creature — Zombie Warrior 3/1
// When this enters, each player sacrifices a creature.
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
            // PB-SFT (CR 701.17a + CR 109.1c): creature-only filter via TargetFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
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
