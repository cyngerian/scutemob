// Olivia's Wrath — {4}{B} Sorcery
// Each non-Vampire creature gets -X/-X until end of turn, where X is the number of
// Vampires you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("olivias-wrath"),
        name: "Olivia's Wrath".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each non-Vampire creature gets -X/-X until end of turn, where X is the number of Vampires you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // CR 613.1c / Layer 7c: -X/-X to all non-Vampire creatures (any controller).
                // CR 608.2h: X is the number of Vampires YOU control, resolved once at spell
                // resolution time. ModifyBothDynamic is substituted into ModifyBoth(-X) in
                // effects/mod.rs before the ContinuousEffect is stored. negate=true converts
                // the positive PermanentCount into a negative delta.
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBothDynamic {
                            amount: Box::new(EffectAmount::PermanentCount {
                                filter: TargetFilter {
                                    has_card_type: Some(CardType::Creature),
                                    has_subtype: Some(SubType("Vampire".to_string())),
                                    ..Default::default()
                                },
                                controller: PlayerTarget::Controller,
                            }),
                            negate: true,
                        },
                        filter: EffectFilter::AllCreaturesExcludingSubtype(SubType("Vampire".to_string())),
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
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
