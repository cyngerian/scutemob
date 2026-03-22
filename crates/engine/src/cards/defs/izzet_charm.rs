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
            targets: vec![
                TargetRequirement::TargetSpellWithFilter(TargetFilter {
                    non_creature: true,
                    ..Default::default()
                }),
                TargetRequirement::TargetCreature,
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Counter target noncreature spell unless its controller pays {2}.
                    // TODO: "unless its controller pays {2}" — conditional counter not in DSL.
                    // Using unconditional counter as approximation (stronger than intended).
                    Effect::CounterSpell {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    // Mode 1: Deal 2 damage to target creature.
                    Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        amount: EffectAmount::Fixed(2),
                    },
                    // Mode 2: Draw two cards, then discard two cards.
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
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
