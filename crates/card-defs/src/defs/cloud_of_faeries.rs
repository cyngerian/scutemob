// Cloud of Faeries — {1}{U}, Creature — Faerie 1/1
// Flying
// When this creature enters, untap up to two lands.
// Cycling {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cloud-of-faeries"),
        name: "Cloud of Faeries".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Faerie"]),
        oracle_text: "Flying\nWhen this creature enters, untap up to two lands.\nCycling {2}"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 115.10 (PB-DX28): "untap up to two lands" is printed with no
            // "target" — a resolution-time UNTARGETED choice, not a declared target
            // (no `TargetRequirement::UpToN` slot). Note: no "you control" in the
            // printed text either, so any land is eligible, not just this player's.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::UntapPermanent {
                    target: EffectTarget::ChosenObject {
                        zone: ChoiceZone::Battlefield,
                        filter: Box::new(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        }),
                        count: 2,
                        up_to: true,
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
        ],
        ..Default::default()
    }
}
