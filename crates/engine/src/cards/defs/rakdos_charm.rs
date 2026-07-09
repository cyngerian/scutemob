// Rakdos Charm — {B}{R} Instant
// Choose one —
// • Exile target player's graveyard.
// • Destroy target artifact.
// • Each creature deals 1 damage to its controller.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rakdos-charm"),
        name: "Rakdos Charm".to_string(),
        mana_cost: Some(ManaCost { black: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Exile target player's graveyard.\n• Destroy target artifact.\n• Each creature deals 1 damage to its controller.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — mode 0 and mode 1 each declare
            // their own single target, LOCAL to that mode. `Spell.targets` is empty. Mode 2
            // has no declared target ("each creature").
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Exile target player's graveyard.
                    // ENGINE-BLOCKED: no Effect variant exiles an entire graveyard zone
                    // (only single-object ExileObject exists). Unrelated to AC4's
                    // per-mode-targeting scope; the target is still correctly required
                    // (TargetPlayer) when this mode is chosen.
                    Effect::Nothing,
                    // Mode 1: Destroy target artifact.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Mode 2: Each creature deals 1 damage to its controller.
                    // ENGINE-BLOCKED: `Effect::DealDamage` takes an `EffectTarget`, which
                    // has no `ControllerOf` variant, so damage cannot be routed to the
                    // controller of the current ForEach iteration object. (`PlayerTarget`
                    // does have `ControllerOf`, but DealDamage does not accept a
                    // `PlayerTarget`.) Unrelated to AC4's per-mode-targeting scope.
                    Effect::Nothing,
                ],
                mode_targets: Some(vec![
                    vec![TargetRequirement::TargetPlayer],
                    vec![TargetRequirement::TargetArtifact],
                    vec![],
                ]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
