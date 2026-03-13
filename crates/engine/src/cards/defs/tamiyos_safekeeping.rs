// Tamiyo's Safekeeping — {G}, Instant
// Target permanent you control gains hexproof and indestructible until end of turn.
// You gain 2 life.
//
// CR 611.3a: Until-end-of-turn continuous effect via ApplyContinuousEffect.
// CR 702.11a: Hexproof; CR 702.12a: Indestructible.
// Uses Sequence to apply the continuous effect and then gain 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tamiyos-safekeeping"),
        name: "Tamiyo's Safekeeping".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target permanent you control gains hexproof and indestructible until end of turn. You gain 2 life. (A permanent with hexproof and indestructible can't be the target of spells or abilities your opponents control. Damage and effects that say \"destroy\" don't destroy it.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
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
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(2),
                    },
                ]),
                targets: vec![TargetRequirement::TargetPermanent],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
