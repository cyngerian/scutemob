// Marang River Regent // Coil and Catch — {4}{U}{U} Creature — Dragon 6/7
// Flying
// When this creature enters, return up to two other target nonland permanents
// to their owners' hands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("marang-river-regent"),
        name: "Marang River Regent // Coil and Catch".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhen this creature enters, return up to two other target nonland permanents to their owners' hands.".to_string(),
        power: Some(6),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 601.2c / 115.1b: "When this creature enters, return up to two other target
            // nonland permanents to their owners' hands." UpToN fixed.
            // Note: "other" (exclude_self) not in TargetFilter; omitted (minor over-permissive:
            // Regent could target itself, which is cosmetically wrong but resolved correctly by
            // MoveZone to own hand). Tracked as LOW.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                        },
                        controller_override: None,
                    },
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 1 })),
                        },
                        controller_override: None,
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::UpToN {
                    count: 2,
                    inner: Box::new(TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        non_land: true,
                        ..Default::default()
                    })),
                }],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
