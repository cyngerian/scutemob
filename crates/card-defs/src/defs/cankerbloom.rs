// Cankerbloom — {1}{G} Creature — Phyrexian Fungus 3/2
// {1}, Sacrifice this creature: Choose one —
// • Destroy target artifact.
// • Destroy target enchantment.
// • Proliferate.
//
// CR 602.2/700.2a/700.2c (PB-EF7): Modal activated ability using
// `AbilityDefinition::Activated::modes` (ModeSelection). The controller chooses the
// mode at activation; the chosen mode's effect is baked into `embedded_effect` at
// activation time (approach (a) — required because the {1}, Sacrifice cost removes
// this creature's ObjectId before resolution, CR 400.7). Mode 2 (Proliferate) has an
// EMPTY target slice (CR 700.2c: "the ability is treated as though it did not have
// those targets" for a mode that isn't chosen) -- activating Proliferate no longer
// requires a legal artifact and enchantment on the battlefield, unlike the old
// Effect::Choose encoding.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cankerbloom"),
        name: "Cankerbloom".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Creature], &["Phyrexian", "Fungus"]),
        oracle_text: "{1}, Sacrifice this creature: Choose one —\n• Destroy target artifact.\n• \
                      Destroy target enchantment.\n• Proliferate. (Choose any number of \
                      permanents and/or players, then give each another counter of each kind \
                      already there.)"
            .to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // {1}, Sacrifice this creature: Choose one —
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 1,
                        ..Default::default()
                    }),
                    Cost::SacrificeSelf,
                ]),
                // Placeholder — the real effect lives per-mode in `modes` below.
                effect: Effect::Sequence(vec![]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: Destroy target artifact.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            cant_be_regenerated: false,
                        },
                        // Mode 1: Destroy target enchantment.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            cant_be_regenerated: false,
                        },
                        // Mode 2: Proliferate. No target.
                        Effect::Proliferate,
                    ],
                    mode_targets: Some(vec![
                        // Mode 0 target: artifact.
                        vec![TargetRequirement::TargetArtifact],
                        // Mode 1 target: enchantment.
                        vec![TargetRequirement::TargetEnchantment],
                        // Mode 2: no target (CR 700.2c).
                        vec![],
                    ]),
                }),
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
