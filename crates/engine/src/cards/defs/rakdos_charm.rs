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
            targets: vec![
                // Mode 0: target player (for graveyard exile)
                TargetRequirement::TargetPlayer,
                // Mode 1: target artifact
                TargetRequirement::TargetArtifact,
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Exile target player's graveyard.
                    // TODO: Effect::ExileGraveyard not in DSL — approximated as Nothing
                    Effect::Nothing,
                    // Mode 1: Destroy target artifact.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                    cant_be_regenerated: false,
                    },
                    // Mode 2: Each creature deals 1 damage to its controller.
                    // TODO: "each creature deals 1 damage to its controller" — no per-creature self-damage effect
                    Effect::Nothing,
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
