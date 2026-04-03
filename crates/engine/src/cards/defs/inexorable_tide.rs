// Simple Proliferate enabler with no ETB targeting.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("inexorable-tide"),
        name: "Inexorable Tide".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever you cast a spell, proliferate. (You choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: None,
                    noncreature_only: false,
                    chosen_subtype_filter: false,
                },
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
