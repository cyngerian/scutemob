// Sea-Dasher Octopus — {1}{U}{U}, Creature — Octopus 2/2
// Mutate {1}{U}
// Flash
// Whenever this creature deals combat damage to a player, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sea-dasher-octopus"),
        name: "Sea-Dasher Octopus".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 2, ..Default::default() }),
        types: creature_types(&["Octopus"]),
        oracle_text: "Mutate {1}{U} (If you cast this spell for its mutate cost, put it over or under target non-Human creature you own. They mutate into the creature on top plus all abilities from under it.)\nFlash\nWhenever this creature deals combat damage to a player, draw a card.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Mutate),
            AbilityDefinition::MutateCost {
                cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
            },
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
