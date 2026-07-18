// Boros Charm — {R}{W} Instant
// Choose one — • Boros Charm deals 4 damage to target player or planeswalker.
// • Permanents you control gain indestructible until end of turn.
// • Target creature gains double strike until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boros-charm"),
        name: "Boros Charm".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Boros Charm deals 4 damage to target player or \
                      planeswalker.\n• Permanents you control gain indestructible until end of \
                      turn.\n• Target creature gains double strike until end of turn."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — mode 0 and mode 2 each declare
            // their own single target, LOCAL to that mode. `Spell.targets` is empty. Mode 1
            // has no targets ("permanents you control" — mass grant, not a declared target).
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: 4 damage to target player or planeswalker.
                    Effect::DealDamage {
                        source: None,
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(4),
                    },
                    // Mode 1: Permanents you control gain indestructible until EOT.
                    // ENGINE-BLOCKED: `EffectFilter`/`ContinuousEffectDef` has no catch-all
                    // "all permanents you control" variant — only type-scoped variants exist
                    // (`CreaturesYouControl`, `ArtifactsYouControl`, `LandsYouControl`, etc.).
                    // A generic "permanents you control" filter would need a new EffectFilter
                    // variant; unrelated to AC4's per-mode-targeting scope.
                    Effect::Nothing,
                    // Mode 2: Target creature gains double strike until EOT.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(
                                KeywordAbility::DoubleStrike,
                            ),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ],
                mode_targets: Some(vec![
                    vec![TargetRequirement::TargetPlayerOrPlaneswalker],
                    vec![],
                    vec![TargetRequirement::TargetCreature],
                ]),
            }),
            cant_be_countered: false,
        }],
        completeness: Completeness::partial(
            "`EffectFilter`/`ContinuousEffectDef` has no catch-all 'all permanents you control' \
             variant — only type-scoped variants...",
        ),
        ..Default::default()
    }
}
