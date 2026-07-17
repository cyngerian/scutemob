// Flusterstorm — {U}, Instant
// Counter target instant or sorcery spell unless its controller pays {1}.
// Storm
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flusterstorm"),
        name: "Flusterstorm".to_string(),
        mana_cost: Some(ManaCost {
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target instant or sorcery spell unless its controller pays \
                      {1}.\nStorm (When you cast this spell, copy it for each spell cast before \
                      it this turn. You may choose new targets for the copies.)"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Storm),
            AbilityDefinition::Spell {
                // PB-AC2 (CR 118.12a): CounterUnlessPays — controller declines -> countered.
                effect: Effect::CounterUnlessPays {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cost: Cost::Mana(ManaCost {
                        generic: 1,
                        ..Default::default()
                    }),
                },
                targets: vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Instant, CardType::Sorcery],
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
