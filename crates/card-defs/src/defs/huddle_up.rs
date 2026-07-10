// Huddle Up — {2}{U}, Sorcery; Assist (Another player may pay up to {2} of this spell's cost.)
// Two target players each draw a card.
// CR 702.132: Assist — another player may pay up to the generic portion of this spell's mana cost.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("huddle-up"),
        name: "Huddle Up".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Assist (Another player may pay up to {2} of this spell's cost.)\nTwo target players each draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Assist),
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::DrawCards {
                        player: PlayerTarget::DeclaredTarget { index: 1 },
                        count: EffectAmount::Fixed(1),
                    },
                ]),
                targets: vec![
                    TargetRequirement::TargetPlayer,
                    TargetRequirement::TargetPlayer,
                ],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
