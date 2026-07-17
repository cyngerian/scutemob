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
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            green: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose one or more —\n• Destroy target artifact.\n• Destroy target \
                      creature.\n• Destroy target enchantment.\n• Destroy target land.\n• Destroy \
                      target planeswalker."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — one target per chosen mode,
            // declared only for the modes actually chosen. `Spell.targets` is empty; each
            // mode's single target lives in `mode_targets` at LOCAL index 0. Before AC4
            // this card's flat target list demanded all five targets be declared
            // regardless of which mode(s) were chosen, making it effectively uncastable
            // in most board states (wrong game state).
            targets: vec![],
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
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Mode 2: Destroy target enchantment.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Mode 3: Destroy target land.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // Mode 4: Destroy target planeswalker.
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                ],
                mode_targets: Some(vec![
                    vec![TargetRequirement::TargetArtifact],
                    vec![TargetRequirement::TargetCreature],
                    vec![TargetRequirement::TargetEnchantment],
                    vec![TargetRequirement::TargetLand],
                    vec![TargetRequirement::TargetPlaneswalker],
                ]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
