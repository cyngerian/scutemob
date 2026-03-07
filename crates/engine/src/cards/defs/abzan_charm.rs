// Abzan Charm — {W}{B}{G} Instant; choose one of three modes:
// 0: Exile target creature with power 3 or greater.
// 1: Draw 2 cards and lose 2 life.
// 2: Distribute two +1/+1 counters among one or two target creatures you control.
//    (Approximated: add 2 counters to a single target creature you control — the DSL does
//    not support distributing counters across two independently-declared targets per mode.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abzan-charm"),
        name: "Abzan Charm".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n\
            • Exile target creature with power 3 or greater.\n\
            • You draw two cards and you lose 2 life.\n\
            • Distribute two +1/+1 counters among one or two target creatures."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // Targets:
            //   index 0: mode 0 — creature with power >= 3
            //   index 1: mode 2 — target creature you control (receives up to 2 counters)
            // Mode 1 has no targets (self-referential draw/lose life).
            // TODO: per-mode target lists are not supported; all targets are declared up front.
            // When mode-scoped targeting is added, mode 0 and mode 2 targets should be separate.
            targets: vec![
                TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    min_power: Some(3),
                    ..Default::default()
                }),
                TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                }),
            ],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Exile target creature with power 3 or greater.
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    // Mode 1: You draw two cards and you lose 2 life.
                    Effect::Sequence(vec![
                        Effect::DrawCards {
                            player: PlayerTarget::Controller,
                            count: EffectAmount::Fixed(2),
                        },
                        Effect::LoseLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                    ]),
                    // Mode 2: Distribute two +1/+1 counters among one or two target creatures.
                    // TODO: distributing counters across two separately-declared targets is not
                    // expressible in the current DSL. This places both counters on a single target
                    // creature you control. When split-counter distribution is added, refactor to
                    // declare two optional creature targets and split AddCounter across them.
                    Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 2,
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
