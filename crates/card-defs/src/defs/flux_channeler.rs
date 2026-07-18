// Flux Channeler — {2}{U}, Creature — Human Wizard 2/2
// Whenever you cast a noncreature spell, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flux-channeler"),
        name: "Flux Channeler".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Whenever you cast a noncreature spell, proliferate. (Choose any number of \
                      permanents and/or players, then give each another counter of each kind \
                      already there.)"
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // Whenever you cast a noncreature spell, proliferate.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: None,
                    noncreature_only: true,
                    chosen_subtype_filter: false,
                    spell_subtype_filter: None,
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
