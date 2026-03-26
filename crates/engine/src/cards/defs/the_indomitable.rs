// The Indomitable — {2}{U}{U}, Legendary Artifact — Vehicle 6/6
// Trample
// Whenever a creature you control deals combat damage to a player, draw a card.
// Crew 3
//
// TODO: "Cast from graveyard if 3+ tapped Pirates/Vehicles" not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-indomitable"),
        name: "The Indomitable".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &["Vehicle"]),
        oracle_text: "Trample\nWhenever a creature you control deals combat damage to a player, draw a card.\nCrew 3\nYou may cast this card from your graveyard as long as you control three or more tapped Pirates and/or Vehicles.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 510.3a: "Whenever a creature you control deals combat damage to a player,
            // draw a card." — per-creature trigger.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: None },
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
            AbilityDefinition::Keyword(KeywordAbility::Crew(3)),
        ],
        ..Default::default()
    }
}
