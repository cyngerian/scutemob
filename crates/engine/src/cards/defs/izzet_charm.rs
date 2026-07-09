// Izzet Charm — {U}{R} Instant
// Choose one —
// • Counter target noncreature spell unless its controller pays {2}.
// • Izzet Charm deals 2 damage to target creature.
// • Draw two cards, then discard two cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("izzet-charm"),
        name: "Izzet Charm".to_string(),
        mana_cost: Some(ManaCost { blue: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Counter target noncreature spell unless its controller pays {2}.\n• Izzet Charm deals 2 damage to target creature.\n• Draw two cards, then discard two cards.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — targets are declared only for
            // the chosen mode. `Spell.targets` is empty; each mode's requirements live in
            // `mode_targets`, and effects use LOCAL (0-based) DeclaredTarget indices.
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Counter target noncreature spell unless its controller pays {2}.
                    // PB-AC2 (CR 118.12a): CounterUnlessPays — controller declines -> countered.
                    Effect::CounterUnlessPays {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    },
                    // Mode 1: Deal 2 damage to target creature.
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(2),
                    },
                    // Mode 2: Draw two cards, then discard two cards. No target.
                    Effect::Sequence(vec![
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                        Effect::DiscardCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                    ]),
                ],
                mode_targets: Some(vec![
                    vec![TargetRequirement::TargetSpellWithFilter(TargetFilter {
                        non_creature: true,
                        ..Default::default()
                    })],
                    vec![TargetRequirement::TargetCreature],
                    vec![],
                ]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
