// Darksteel Garrison — {2}, Artifact — Fortification; Future Sight
// Fortified land has indestructible.
// Whenever fortified land becomes tapped, target creature gets +1/+1 until end of turn.
// Fortify {3} (CR 702.67a)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("darksteel-garrison"),
        name: "Darksteel Garrison".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Fortification"]),
        oracle_text: "Fortified land has indestructible.\nWhenever fortified land becomes tapped, \
                      target creature gets +1/+1 until end of turn.\nFortify {3} ({3}: Attach to \
                      target land you control. Fortify only as a sorcery. This card enters \
                      unattached and stays on the battlefield if the land leaves.)"
            .to_string(),
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
                cost: Cost::Mana(ManaCost {
                    generic: 3,
                    ..Default::default()
                }),
                effect: Effect::AttachFortification {
                    fortification: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-1) / CR 702.67a: "Fortify {3}" means "[Cost]: Attach
                // this permanent to target LAND you control." Printed line MCP-verified as
                // plain "Fortify {3}" with no further quality restriction.
                //
                // NOT the equip repair's `TargetCreatureWithFilter`: copying CARDS-1's shape
                // verbatim would demand a *creature* and make this ability un-activatable on
                // the only permanents it may legally attach to. There is no controller-scoped
                // land analogue (`TargetRequirement::TargetLand` is unfiltered), so the
                // "you control" half comes from the filter's own `controller` field —
                // `casting.rs`'s `TargetPermanentWithFilter` arm checks `has_card_type` via
                // `matches_filter` and `TargetController::You => obj.controller == caster`
                // in the same arm.
                //
                // Before this: `targets: vec![]` with the effect reading
                // `DeclaredTarget { index: 0 }` — the offer layer reported zero slots, so
                // nothing ever asked, the cost was paid, and the attach fizzled in silence.
                // Identical shape and identical chain to the equip defect CARDS-1 closed.
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_type: Some(CardType::Land),
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::partial(
            "TriggerCondition::WhenFortifiedLandBecomesTapped does not exist yet, so 'Whenever \
             fortified land becomes tapped, target creature gets +1/+1 until end of turn' is \
             unimplemented — the trigger never fires and no creature is ever pumped (re-checked \
             against the current enum 2026-08-11; WhenSelfBecomesTapped is self-scoped to the \
             Fortification, not the fortified land). The indestructible static \
             (EffectFilter::AttachedLand) and Fortify {3} — including its CR 702.67a target, \
             authored by PB-DX26 / OOS-CARDS1-1 — ARE implemented.",
        ),
        ..Default::default()
    }
}
