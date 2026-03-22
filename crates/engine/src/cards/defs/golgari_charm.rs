// Golgari Charm — {B}{G} Instant; choose one of three modes:
// 0: All creatures get -1/-1 until end of turn.
// 1: Destroy target enchantment.
// 2: Regenerate each creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("golgari-charm"),
        name: "Golgari Charm".to_string(),
        mana_cost: Some(ManaCost { black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n\
            • All creatures get -1/-1 until end of turn.\n\
            • Destroy target enchantment.\n\
            • Regenerate each creature you control."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // Mode 1: target enchantment (index 0). Modes 0 and 2 have no declared targets.
            targets: vec![TargetRequirement::TargetEnchantment],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: All creatures get -1/-1 until end of turn.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyBoth(-1),
                            filter: EffectFilter::AllCreatures,
                            duration: EffectDuration::UntilEndOfTurn,
                        }),
                    },
                    // Mode 1: Destroy target enchantment.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                    },
                    // Mode 2: Regenerate each creature you control.
                    // TODO: Effect::RegenerateAll (or bulk regenerate for your creatures) does
                    // not exist in the DSL. When a regenerate-all effect is added, replace this
                    // with the correct Effect variant filtered to CreaturesYouControl.
                    Effect::Sequence(vec![]),
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
