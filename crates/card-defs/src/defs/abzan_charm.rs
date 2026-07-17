// Abzan Charm — {W}{B}{G} Instant; choose one of three modes:
// 0: Exile target creature with power 3 or greater.
// 1: Draw 2 cards and lose 2 life.
// 2: Distribute two +1/+1 counters among one or two target creatures. (No controller
//    restriction — any creature is a legal target.)
//    (ENGINE-BLOCKED: approximated — add 2 counters to a single target creature. See
//    inline comment on mode 2 below for the missing distribute-N-among-M primitive.)
//
// PB-AC4 (CR 700.2c/700.2f): per-mode targets migrated — modes 0 and 2 each declare their
// own single target via `mode_targets`, LOCAL to that mode (`Spell.targets` is empty).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("abzan-charm"),
        name: "Abzan Charm".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Exile target creature with power 3 or greater.\n• You draw \
                      two cards and you lose 2 life.\n• Distribute two +1/+1 counters among one \
                      or two target creatures."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — mode 0 and mode 2 each declare
            // their own single target, LOCAL to that mode. `Spell.targets` is empty. Mode 1
            // has no targets (self-referential draw/lose life).
            targets: vec![],
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
                    // ENGINE-BLOCKED: distributing a fixed counter pool across 1-2
                    // independently-declared targets needs a distribute-N-among-M-targets
                    // primitive that does not exist in the DSL (unrelated to AC4's
                    // per-mode-targeting scope). Approximated: both counters go on a single
                    // declared target creature (no controller restriction — any creature is
                    // a legal target per oracle text).
                    Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 2,
                    },
                ],
                mode_targets: Some(vec![
                    vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        min_power: Some(3),
                        ..Default::default()
                    })],
                    vec![],
                    vec![TargetRequirement::TargetCreature],
                ]),
            }),
            cant_be_countered: false,
        }],
        completeness: Completeness::partial(
            "approximated — add 2 counters to a single target creature. See inline comment on \
             mode 2 below for the missing...",
        ),
        ..Default::default()
    }
}
