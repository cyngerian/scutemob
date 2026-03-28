// Baleful Strix — {U}{B}, Artifact Creature — Bird 1/1
// Flying, deathtouch
// When this creature enters, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("baleful-strix"),
        name: "Baleful Strix".to_string(),
        mana_cost: Some(ManaCost { blue: 1, black: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Artifact, CardType::Creature], &["Bird"]),
        oracle_text: "Flying, deathtouch\nWhen this creature enters, draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
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
