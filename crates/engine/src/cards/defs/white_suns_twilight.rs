// White Sun's Twilight — {X}{W}{W}, Sorcery
// You gain X life. Create X 1/1 colorless Phyrexian Mite artifact creature tokens with
// toxic 1 and "This token can't block." If X is 5 or more, destroy all other creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("white-suns-twilight"),
        name: "White Sun's Twilight".to_string(),
        mana_cost: Some(ManaCost { white: 2, x_count: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You gain X life. Create X 1/1 colorless Phyrexian Mite artifact creature tokens with toxic 1 and \"This token can't block.\" If X is 5 or more, destroy all other creatures. (Players dealt combat damage by a creature with toxic 1 also get a poison counter.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 107.3m: Gain X life.
                Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::XValue,
                },
                // CR 107.3m: Create X Phyrexian Mite tokens.
                // Note: "This token can't block" is a static restriction; KeywordAbility::CantBlock
                // does not exist in DSL — omitted per existing practice.
                Effect::Repeat {
                    count: EffectAmount::XValue,
                    effect: Box::new(Effect::CreateToken {
                        spec: TokenSpec {
                            name: "Phyrexian Mite".to_string(),
                            card_types: [CardType::Artifact, CardType::Creature].into_iter().collect(),
                            subtypes: [SubType("Phyrexian".to_string()), SubType("Mite".to_string())].into_iter().collect(),
                            colors: im::OrdSet::new(),
                            supertypes: im::OrdSet::new(),
                            power: 1,
                            toughness: 1,
                            count: 1,
                            keywords: [KeywordAbility::Toxic(1)].into_iter().collect(),
                            tapped: false,
                            enters_attacking: false,
                            mana_color: None,
                            mana_abilities: vec![],
                            activated_abilities: vec![],
                            ..Default::default()
                        },
                    }),
                },
                // CR 107.3m: "If X is 5 or more, destroy all other creatures."
                Effect::Conditional {
                    condition: Condition::XValueAtLeast(5),
                    if_true: Box::new(Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                        cant_be_regenerated: false,
                    }),
                    if_false: Box::new(Effect::Nothing),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
