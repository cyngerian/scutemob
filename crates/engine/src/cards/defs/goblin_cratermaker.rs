// Goblin Cratermaker — {1}{R} Creature — Goblin Warrior 2/2
// {1}, Sacrifice this creature: Choose one —
// • This creature deals 2 damage to target creature.
// • Destroy target colorless nonland permanent.
//
// CR 602.2 / PB-35: Modal activated ability using Effect::Choose (bot auto-picks mode 0).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-cratermaker"),
        name: "Goblin Cratermaker".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "{1}, Sacrifice this creature: Choose one —\n• Goblin Cratermaker deals 2 damage to target creature.\n• Destroy target colorless nonland permanent.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 602.2 / PB-35: {1}, Sacrifice: Choose one — 2 damage OR destroy colorless.
            // Modal activated ability expressed via Effect::Choose (auto-picks mode 0 = damage).
            // Mode 0: Deal 2 damage to target creature.
            // Mode 1: Destroy target colorless nonland permanent.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Choose {
                    prompt: "Choose one — deal 2 damage to target creature; or destroy target colorless nonland permanent".to_string(),
                    choices: vec![
                        // Mode 0: Goblin Cratermaker deals 2 damage to target creature.
                        Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(2),
                        },
                        // Mode 1: Destroy target colorless nonland permanent.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 1 },
                            cant_be_regenerated: false,
                        },
                    ],
                },
                timing_restriction: None,
                targets: vec![
                    // Mode 0 target: any creature (index 0)
                    TargetRequirement::TargetCreature,
                    // Mode 1 target: colorless nonland permanent (index 1).
                    // Note: colorless filter not expressible in TargetFilter; non_land used as
                    // approximation (any nonland permanent). Bot picks first nonland permanent.
                    TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        non_land: true,
                        ..Default::default()
                    }),
                ],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
