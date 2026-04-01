// Whitemane Lion — {1}{W}, Creature — Cat 2/1
// Flash.
// When this enters, return a creature you control to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("whitemane-lion"),
        name: "Whitemane Lion".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Cat"]),
        oracle_text: "Flash\nWhen this enters, return a creature you control to its owner's hand.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // CR 603.3: ETB trigger — return a creature you control to its owner's hand.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                    },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
