// Heroic Intervention — {1}{G}, Instant.
// "Creatures you control gain hexproof and indestructible until end of turn."
// CR 611.3a: Continuous effect applied until end of turn (cleanup step).
// Uses ForEach over EachCreatureYouControl; each creature gets a SingleObject
// continuous effect via DeclaredTarget { index: 0 } resolved at execution time.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("heroic-intervention"),
        name: "Heroic Intervention".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Creatures you control gain hexproof and indestructible until end of turn.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCreatureYouControl,
                    effect: Box::new(Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeywords(
                                [KeywordAbility::Hexproof, KeywordAbility::Indestructible]
                                    .into_iter()
                                    .collect(),
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
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
