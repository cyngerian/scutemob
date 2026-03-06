// Path of Peril — {1}{B}{B} Sorcery; Cleave {4}{W}{B}; destroy all creatures [with mana value 2 or less]
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("path-of-peril"),
        name: "Path of Peril".to_string(),
        // MCP oracle: {1}{B}{B} (user prompt had {1}{W}{B}; oracle text is authoritative)
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Cleave {4}{W}{B} (You may cast this spell for its cleave cost. If you do, remove the words in square brackets.)\nDestroy all creatures [with mana value 2 or less].".to_string(),
        abilities: vec![
            // CR 702.148a: Cleave keyword marker for presence-checking
            AbilityDefinition::Keyword(KeywordAbility::Cleave),
            // CR 702.148a: Cleave alternative cost declaration
            AbilityDefinition::Cleave {
                cost: ManaCost { generic: 4, white: 1, black: 1, ..Default::default() },
            },
            // Spell effect: branch on WasCleaved (CR 702.148)
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasCleaved,
                    // Cleaved: destroy ALL creatures (no restriction)
                    if_true: Box::new(Effect::DestroyPermanent {
                        target: EffectTarget::AllCreatures,
                    }),
                    // Normal cast: destroy all creatures with mana value 2 or less
                    // TODO: TargetFilter lacks max_mana_value field; AllPermanentsMatching
                    // only filters by creature type, not mana value. This over-approximates
                    // by destroying ALL creatures when cast normally. A proper implementation
                    // requires adding `max_mana_value: Option<u32>` to TargetFilter.
                    if_false: Box::new(Effect::DestroyPermanent {
                        target: EffectTarget::AllPermanentsMatching(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        }),
                    }),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
