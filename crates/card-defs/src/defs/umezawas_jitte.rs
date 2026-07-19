// Umezawa's Jitte — {2}, Legendary Artifact — Equipment
// Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte.
// Remove a charge counter from Umezawa's Jitte: Choose one —
// • Equipped creature gets +2/+2 until end of turn.
// • Target creature gets -1/-1 until end of turn.
// • You gain 2 life.
// Equip {2}
//
// PB-OS10 (2026-07-19, OOS-EF7-1): the counters trigger uses the any-recipient
// `WhenEquippedCreatureDealsCombatDamage` variant (player, creature, or planeswalker —
// not just players), and the counter-removal ability is a real modal activated ability
// (`AbilityDefinition::Activated::modes`, the PB-EF7 primitive). Execution-verified:
// any-recipient trigger, `Cost::RemoveCounter` payment/gating, and all three modes.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("umezawas-jitte"),
        name: "Umezawa's Jitte".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Whenever equipped creature deals combat damage, put two charge counters on \
                      Umezawa's Jitte.\nRemove a charge counter from Umezawa's Jitte: Choose one \
                      —\n• Equipped creature gets +2/+2 until end of turn.\n• Target creature \
                      gets -1/-1 until end of turn.\n• You gain 2 life.\nEquip {2}"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // CR 510.3a: Whenever equipped creature deals combat damage, put two charge counters
            // on Umezawa's Jitte.
            // PB-OS10 (OOS-EF7-1): oracle says "deals combat damage" (any recipient, not just
            // players) — now uses the any-recipient WhenEquippedCreatureDealsCombatDamage variant.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamage,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Charge,
                    count: 2,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 602.2 / 700.2a (PB-OS10): Remove a charge counter: Choose one —
            // Mode 0: Equipped creature gets +2/+2 until end of turn.
            // Mode 1: Target creature gets -1/-1 until end of turn.
            // Mode 2: You gain 2 life.
            AbilityDefinition::Activated {
                cost: Cost::RemoveCounter {
                    counter: CounterType::Charge,
                    count: 1,
                },
                effect: Effect::Sequence(vec![]), // placeholder; real effects live in `modes`
                timing_restriction: None,
                targets: vec![], // MUST be empty when mode_targets is Some
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 1,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: equipped creature +2/+2 EOT (no target)
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyBoth(2),
                                filter: EffectFilter::AttachedCreature,
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                        // Mode 1: target creature -1/-1 EOT (1 target)
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyBoth(-1),
                                filter: EffectFilter::DeclaredTarget { index: 0 },
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                        // Mode 2: gain 2 life (no target)
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                    ],
                    mode_targets: Some(vec![
                        vec![],                                  // Mode 0: no targets
                        vec![TargetRequirement::TargetCreature], // Mode 1: one target creature
                        vec![],                                  // Mode 2: no targets
                    ]),
                }),
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
