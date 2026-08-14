// Shrieking Drake — {U}, Creature — Drake 1/1
// "Flying
// When this creature enters, return a creature you control to its owner's hand."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shrieking-drake"),
        name: "Shrieking Drake".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Drake"]),
        oracle_text: "Flying\nWhen this creature enters, return a creature you control to its \
                      owner's hand."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.1 / CR 115.10 (PB-DX28): When this creature enters, return a
            // creature you control to its owner's hand. Printed with no "target" — a
            // resolution-time UNTARGETED choice (CR 115.10), not a declared target:
            // unaffected by hexproof/shroud/protection, and re-chosen at resolution if
            // the original candidate leaves in response (no CR 608.2b fizzle window).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::ChosenObject {
                        zone: ChoiceZone::Battlefield,
                        filter: Box::new(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            controller: TargetController::You,
                            ..Default::default()
                        }),
                        count: 1,
                        up_to: false,
                    },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::ChosenObject {
                            zone: ChoiceZone::Battlefield,
                            filter: Box::new(TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                controller: TargetController::You,
                                ..Default::default()
                            }),
                            count: 1,
                            up_to: false,
                        })),
                    },
                    controller_override: None,
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
