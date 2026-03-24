// Darksteel Garrison — {2}, Artifact — Fortification; Future Sight
// Fortified land has indestructible.
// Whenever fortified land becomes tapped, target creature gets +1/+1 until end of turn.
// Fortify {3} (CR 702.67a)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darksteel-garrison"),
        name: "Darksteel Garrison".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Fortification"]),
        oracle_text: "Fortified land has indestructible.\nWhenever fortified land becomes tapped, target creature gets +1/+1 until end of turn.\nFortify {3} ({3}: Attach to target land you control. Fortify only as a sorcery. This card enters unattached and stays on the battlefield if the land leaves.)".to_string(),
        abilities: vec![
            // CR 604.2 / CR 702.67: Static ability — fortified land has indestructible (layer 6).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeywords(
                        [KeywordAbility::Indestructible].into_iter().collect(),
                    ),
                    filter: EffectFilter::AttachedLand,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: TriggerCondition::WhenFortifiedLandBecomesTapped does not exist yet.
            // "Whenever fortified land becomes tapped, target creature gets +1/+1 until end of turn."
            // Cannot be expressed in the current DSL. Add WhenFortifiedLandBecomesTapped
            // to TriggerCondition and a corresponding +1/+1 counter/buff effect when the
            // variant is implemented.

            // CR 702.67a: Fortify {3} — activated ability; sorcery speed.
            AbilityDefinition::Keyword(KeywordAbility::Fortify),
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                effect: Effect::AttachFortification {
                    fortification: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
