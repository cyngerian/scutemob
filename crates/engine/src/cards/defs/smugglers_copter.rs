// 67. Smuggler's Copter — {2}, Artifact — Vehicle 3/3; Flying; Crew 1;
// Whenever it attacks or blocks, you may draw a card. If you do, discard a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("smugglers-copter"),
        name: "Smuggler's Copter".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: TypeLine {
            card_types: [CardType::Artifact].iter().copied().collect(),
            subtypes: ["Vehicle".to_string()].iter().cloned().map(SubType).collect(),
            ..Default::default()
        },
        oracle_text: "Flying\nCrew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)\nWhenever Smuggler's Copter attacks or blocks, you may draw a card. If you do, discard a card.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Crew(1)),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                    Effect::DiscardCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                ]),
                intervening_if: None,
            },
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenBlocks,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                    Effect::DiscardCards { player: PlayerTarget::Controller, count: EffectAmount::Fixed(1) },
                ]),
                intervening_if: None,
            },
        ],
        color_indicator: None,
        back_face: None,
    }
}
