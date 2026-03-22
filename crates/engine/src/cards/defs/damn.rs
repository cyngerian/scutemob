// Damn — {B}{B} Sorcery
// Destroy target creature. A creature destroyed this way can't be regenerated.
// Overload {2}{W}{W} (You may cast this spell for its overload cost. If you do,
// change "target" in its text to "each.")
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("damn"),
        name: "Damn".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Destroy target creature. A creature destroyed this way can't be regenerated.\nOverload {2}{W}{W} (You may cast this spell for its overload cost. If you do, change \"target\" in its text to \"each.\")".to_string(),
        abilities: vec![
            // CR 702.96a: Overload keyword marker.
            AbilityDefinition::Keyword(KeywordAbility::Overload),
            // CR 702.96a: Overload alternative cost {2}{W}{W}.
            AbilityDefinition::Overload {
                cost: ManaCost { generic: 2, white: 2, ..Default::default() },
            },
            // CR 702.96b: Spell effect — branches on WasOverloaded.
            // Normal: destroy the declared target creature (can't be regenerated).
            // Overloaded: destroy each creature (can't be regenerated).
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasOverloaded,
                    if_true: Box::new(Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            ..Default::default()
                        },
                        cant_be_regenerated: true,
                    }),
                    if_false: Box::new(Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: true,
                    }),
                },
                // Normal cast: target one creature.
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
