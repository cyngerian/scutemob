// Casualties of War — {2}{B}{B}{G}{G} Sorcery
// Choose one or more —
// • Destroy target artifact.
// • Destroy target creature.
// • Destroy target enchantment.
// • Destroy target land.
// • Destroy target planeswalker.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("casualties-of-war"),
        name: "Casualties of War".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one or more —\n\
            • Destroy target artifact.\n\
            • Destroy target creature.\n\
            • Destroy target enchantment.\n\
            • Destroy target land.\n\
            • Destroy target planeswalker."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // Each mode has one declared target. Indices map to chosen modes.
            // All five targets are declared up front; the DSL does not support per-mode
            // target lists. When mode-scoped targeting is added, each mode should declare
            // only its own single target.
            targets: vec![
                TargetRequirement::TargetArtifact,       // mode 0
                TargetRequirement::TargetCreature,        // mode 1
                TargetRequirement::TargetEnchantment,     // mode 2
                TargetRequirement::TargetLand,            // mode 3
                TargetRequirement::TargetPlaneswalker,    // mode 4
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 5,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Destroy target artifact.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                    },
                    // Mode 1: Destroy target creature.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                    cant_be_regenerated: false,
                    },
                    // Mode 2: Destroy target enchantment.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 2 },
                    cant_be_regenerated: false,
                    },
                    // Mode 3: Destroy target land.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 3 },
                    cant_be_regenerated: false,
                    },
                    // Mode 4: Destroy target planeswalker.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 4 },
                    cant_be_regenerated: false,
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
