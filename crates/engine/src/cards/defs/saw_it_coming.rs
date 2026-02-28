// 73. Saw It Coming — {2}{U}{U}, Instant; Counter target spell.
// Foretell {1}{U} (During your turn, you may pay {2} and exile this card from
// your hand face down. Cast it on a future turn for its foretell cost.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("saw-it-coming"),
        name: "Saw It Coming".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell.\nForetell {1}{U} (During your turn, you may pay {2} and exile this card from your hand face down. Cast it on a future turn for its foretell cost.)".to_string(),
        abilities: vec![
            // CR 702.143a: Foretell keyword marker.
            AbilityDefinition::Keyword(KeywordAbility::Foretell),
            // CR 702.143a: Foretell cost ({1}{U}).
            AbilityDefinition::Foretell {
                cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
            },
            // Spell effect: counter target spell.
            AbilityDefinition::Spell {
                effect: Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                targets: vec![TargetRequirement::TargetSpell],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
