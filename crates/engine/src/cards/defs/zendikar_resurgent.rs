// Zendikar Resurgent — {5}{G}{G}, Enchantment
// Whenever you tap a land for mana, add one mana of any type that land produced.
// Whenever you cast a creature spell, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zendikar-resurgent"),
        name: "Zendikar Resurgent".to_string(),
        mana_cost: Some(ManaCost { generic: 5, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever you tap a land for mana, add one mana of any type that land produced. (The types of mana are white, blue, black, red, green, and colorless.)\nWhenever you cast a creature spell, draw a card.".to_string(),
        abilities: vec![
            // CR 605.1b / CR 106.12a: "Whenever you tap a land for mana, add one mana of
            // any type that land produced."
            // Triggered mana ability — resolves immediately (CR 605.4a).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenTappedForMana {
                    source_filter: ManaSourceFilter::Land,
                },
                effect: Effect::AddManaMatchingType {
                    player: PlayerTarget::Controller,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 603.1: "Whenever you cast a creature spell, draw a card."
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                    chosen_subtype_filter: false,
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
