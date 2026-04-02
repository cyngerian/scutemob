// Vandalblast — {R}, Sorcery (Return to Ravnica).
// Normal: Destroy target artifact you don't control.
// Overloaded {4}{R}: Destroy each artifact you don't control.
//
// CR 702.96a: Overload — alternative cost; when paid, "target" becomes "each."
// Modeled via Condition::WasOverloaded branching in the Spell effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vandalblast"),
        name: "Vandalblast".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy target artifact you don't control.\nOverload {4}{R} (You may cast this spell for its overload cost. If you do, change \"target\" in its text to \"each.\")".to_string(),
        abilities: vec![
            // CR 702.96a: Keyword marker for quick presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Overload),
            // CR 702.96a: Overload alternative cost {4}{R}.
            AbilityDefinition::Overload {
                cost: ManaCost { generic: 4, red: 1, ..Default::default() },
            },
            // CR 702.96b: Spell effect — branches on WasOverloaded.
            // Normal: destroy the declared target artifact you don't control.
            // Overloaded: destroy each artifact controlled by an opponent.
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasOverloaded,
                    if_true: Box::new(Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            controller: TargetController::Opponent,
                            ..Default::default()
                        })),
                        effect: Box::new(Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                        }),
                    }),
                    if_false: Box::new(Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                    }),
                },
                // Normal cast: target one artifact you don't control.
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_type: Some(CardType::Artifact),
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
