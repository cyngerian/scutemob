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
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — mode 1 declares its target,
            // LOCAL to that mode. `Spell.targets` is empty. Modes 0 and 2 have no targets.
            targets: vec![],
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
                            condition: None,
                        }),
                    },
                    // Mode 1: Destroy target enchantment.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Mode 2: Regenerate each creature you control.
                    // ENGINE-BLOCKED: no bulk-regenerate Effect variant exists (a single
                    // `Effect::Regenerate` only targets one object). Unrelated to AC4's
                    // per-mode-targeting scope.
                    Effect::Sequence(vec![]),
                ],
                mode_targets: Some(vec![vec![], vec![TargetRequirement::TargetEnchantment], vec![]]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
