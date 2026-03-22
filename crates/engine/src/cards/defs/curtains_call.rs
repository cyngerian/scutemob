// Curtains' Call — {5}{B}, Instant
// Undaunted (This spell costs {1} less to cast for each opponent.)
// Destroy two target creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("curtains-call"),
        name: "Curtains' Call".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Undaunted (This spell costs {1} less to cast for each opponent.)\nDestroy two target creatures.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Undaunted),
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                    },
                ]),
                targets: vec![
                    TargetRequirement::TargetCreature,
                    TargetRequirement::TargetCreature,
                ],
                modes: None,
                cant_be_countered: false,
            },
        ],
        self_cost_reduction: Some(SelfCostReduction::PerOpponent),
        ..Default::default()
    }
}
