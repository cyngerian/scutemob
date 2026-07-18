// Goblin Cratermaker — {1}{R} Creature — Goblin Warrior 2/2
// {1}, Sacrifice this creature: Choose one —
// • This creature deals 2 damage to target creature.
// • Destroy target colorless nonland permanent.
//
// CR 602.2/700.2a (PB-EF7): Modal activated ability using
// `AbilityDefinition::Activated::modes` (ModeSelection). The controller chooses the
// mode at activation; the chosen mode's effect is baked into `embedded_effect` at
// activation time (approach (a) — required because the {1}, Sacrifice cost removes
// this creature's ObjectId before resolution, CR 400.7). Per-mode targets ride
// `ModeSelection.mode_targets` (PB-AC4); each mode's `DeclaredTarget { index: 0 }` is
// LOCAL to that mode's own (single-target) slice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-cratermaker"),
        name: "Goblin Cratermaker".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "{1}, Sacrifice this creature: Choose one —\n• This creature deals 2 damage \
                      to target creature.\n• Destroy target colorless nonland permanent."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 602.2/700.2a: {1}, Sacrifice: Choose one — 2 damage OR destroy colorless.
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
                        // Mode 0: Goblin Cratermaker deals 2 damage to target creature.
                        Effect::DealDamage {
                            source: None,
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(2),
                        },
                        // Mode 1: Destroy target colorless nonland permanent.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            cant_be_regenerated: false,
                        },
                    ],
                    mode_targets: Some(vec![
                        // Mode 0 target: any creature.
                        vec![TargetRequirement::TargetCreature],
                        // Mode 1 target: colorless nonland permanent. "Colorless" is
                        // expressed as excluding all five colors (an object with no
                        // colors passes exclude_colors trivially); non_land rejects lands.
                        vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                            non_land: true,
                            exclude_colors: Some(
                                [
                                    Color::White,
                                    Color::Blue,
                                    Color::Black,
                                    Color::Red,
                                    Color::Green,
                                ]
                                .into_iter()
                                .collect(),
                            ),
                            ..Default::default()
                        })],
                    ]),
                }),
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
