// Ram Through — {1}{G}, Instant
// Target creature you control deals damage equal to its power to target creature you
// don't control. If the creature you control has trample, excess damage is dealt to
// that creature's controller instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ram-through"),
        name: "Ram Through".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature you control deals damage equal to its power to target creature you don't control. If the creature you control has trample, excess damage is dealt to that creature's controller instead.".to_string(),
        abilities: vec![
            // CR 701.14 (one-sided Bite): target[0] (you control) deals damage equal to its
            // power to target[1] (opponent controls). Only one creature deals damage (no fight).
            // TODO: The trample excess-damage clause ("if the creature you control has trample,
            // excess damage is dealt to that creature's controller instead") is a Ram Through-
            // specific behavior not expressible in the current DSL — deferred to card authoring phase.
            AbilityDefinition::Spell {
                effect: Effect::Bite {
                    source: EffectTarget::DeclaredTarget { index: 0 },
                    target: EffectTarget::DeclaredTarget { index: 1 },
                },
                targets: vec![
                    TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    }),
                ],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
