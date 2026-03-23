// Fell Specter — {3}{B}, Creature — Specter 1/3
// Flying
// When this creature enters, target opponent discards a card.
// Whenever an opponent discards a card, that player loses 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fell-specter"),
        name: "Fell Specter".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Specter"]),
        oracle_text: "Flying\nWhen this creature enters, target opponent discards a card.\nWhenever an opponent discards a card, that player loses 2 life.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // ETB: target opponent discards a card.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DiscardCards {
                    player: PlayerTarget::DeclaredTarget { index: 0 },
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPlayer],
            },
            // TODO: "Whenever an opponent discards a card, that player loses 2 life."
            // Requires TriggerCondition::WheneverOpponentDiscards which does not exist in the DSL.
            // The "that player" reference also needs discard trigger target passing.
            // This ability omitted per W5 policy.
        ],
        ..Default::default()
    }
}
