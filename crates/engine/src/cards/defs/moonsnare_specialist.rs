// Moonsnare Specialist — {3}{U}, Creature — Human Ninja 2/2
// Ninjutsu {2}{U}
// When this creature enters, return up to one target creature to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("moonsnare-specialist"),
        name: "Moonsnare Specialist".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {2}{U} ({2}{U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhen this creature enters, return up to one target creature to its owner's hand.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 702.49a: Ninjutsu keyword marker.
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 2, blue: 1, ..Default::default() },
            },
            // CR 603.1 / CR 601.2c / 115.1b: ETB trigger — return up to one target creature
            // to its owner's hand. "Up to one" means the target is optional.
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
                targets: vec![TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetCreature),
                }],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
