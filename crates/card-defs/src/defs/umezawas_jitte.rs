// Umezawa's Jitte — {2}, Legendary Artifact — Equipment
// Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte.
// Remove a charge counter from Umezawa's Jitte: Choose one —
// • Equipped creature gets +2/+2 until end of turn.
// • Target creature gets -1/-1 until end of turn.
// • You gain 2 life.
// Equip {2}
//
// Note: the counter-removal ability below is a "Choose one" modal activated ability;
// PB-EF7 (2026-07-18) shipped the modal-activated-ability primitive it needs
// (`AbilityDefinition::Activated::modes`), but this card is NOT flipped to use it — see
// the `known_wrong` note and OOS-EF7-1 below. The surviving blocker is the counters
// trigger, not the mode selection.
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
            // OOS-EF7-1: Oracle says "deals combat damage" (any recipient), not just to players.
            // Needs a WhenEquippedCreatureDealsCombatDamage variant. Current trigger is the
            // closest available approximation (misses damage dealt to creatures) — this is the
            // real, still-open blocker; see the `known_wrong` note below.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
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
            // CR 602.2 / PB-35: Remove a charge counter: Choose one —
            // Mode 0: Equipped creature gets +2/+2 until end of turn.
            // Mode 1: Target creature gets -1/-1 until end of turn.
            // Mode 2: You gain 2 life.
            // Bot fallback: mode 0 (+2/+2 to equipped creature).
            AbilityDefinition::Activated {
                cost: Cost::RemoveCounter {
                    counter: CounterType::Charge,
                    count: 1,
                },
                effect: Effect::Choose {
                    prompt: "Choose one — equipped creature gets +2/+2; or target creature gets \
                             -1/-1; or you gain 2 life"
                        .to_string(),
                    choices: vec![
                        // Mode 0: Equipped creature gets +2/+2 until end of turn.
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyBoth(2),
                                filter: EffectFilter::AttachedCreature,
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                        // Mode 1: Target creature gets -1/-1 until end of turn.
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::PtModify,
                                modification: LayerModification::ModifyBoth(-1),
                                filter: EffectFilter::DeclaredTarget { index: 0 },
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                        // Mode 2: You gain 2 life.
                        Effect::GainLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        },
                    ],
                },
                timing_restriction: None,
                targets: vec![
                    // Mode 1 target: any creature
                    TargetRequirement::TargetCreature,
                ],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "PB-EF7 (2026-07-18): the modal-activated-ability primitive now exists \
             (AbilityDefinition::Activated::modes / ModeSelection) and would fix the +2/+2-only \
             defect on the counter-removal ability, but that is not the surviving blocker \
             (OOS-EF7-1). The counters trigger fires only on combat damage to PLAYERS — oracle \
             says 'deals combat damage' (any recipient, e.g. a blocking creature) — and needs a \
             WhenEquippedCreatureDealsCombatDamage trigger variant (distinct from \
             WhenEquippedCreatureDealsCombatDamageToPlayer) before this card can be Complete. Not \
             fixed by this PB; scope was 2 flips (Goblin Cratermaker, Cankerbloom) plus this \
             honest note.",
        ),
        ..Default::default()
    }
}
