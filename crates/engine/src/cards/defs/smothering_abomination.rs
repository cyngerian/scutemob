// Smothering Abomination — {2}{B}{B}, Creature — Eldrazi 4/3
// Devoid
// Flying
// At the beginning of your upkeep, sacrifice a creature.
// Whenever you sacrifice a creature, draw a card.
//
// TODO: "Whenever you sacrifice a creature" trigger not in DSL.
// TODO: Forced sacrifice on upkeep not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("smothering-abomination"),
        name: "Smothering Abomination".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, ..Default::default() }),
        types: creature_types(&["Eldrazi"]),
        oracle_text: "Devoid\nFlying\nAt the beginning of your upkeep, sacrifice a creature.\nWhenever you sacrifice a creature, draw a card.".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Devoid),
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "At the beginning of your upkeep, sacrifice a creature" — forced sacrifice not expressible.
            // Whenever you sacrifice a creature, draw a card.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouSacrifice {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                    player_filter: None,
                },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
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
