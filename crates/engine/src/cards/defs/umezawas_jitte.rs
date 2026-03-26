// Umezawa's Jitte — {2}, Legendary Artifact — Equipment
// Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte.
// Remove a charge counter from Umezawa's Jitte: Choose one —
// • Equipped creature gets +2/+2 until end of turn.
// • Target creature gets -1/-1 until end of turn.
// • You gain 2 life.
// Equip {2}
//
// Note: "Choose one" modal activated ability — only the +2/+2 mode is implemented.
// Full modal support (AddCounter on target, GainLife) deferred to PB-37.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("umezawas-jitte"),
        name: "Umezawa's Jitte".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Whenever equipped creature deals combat damage, put two charge counters on Umezawa's Jitte.\nRemove a charge counter from Umezawa's Jitte: Choose one —\n• Equipped creature gets +2/+2 until end of turn.\n• Target creature gets -1/-1 until end of turn.\n• You gain 2 life.\nEquip {2}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // CR 510.3a: Whenever equipped creature deals combat damage, put two charge counters
            // on Umezawa's Jitte.
            // TODO(PB-37): Oracle says "deals combat damage" (any target), not just to players.
            // Needs WhenEquippedCreatureDealsCombatDamage variant. Current trigger is the closest
            // available approximation (misses damage dealt to creatures).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEquippedCreatureDealsCombatDamageToPlayer,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Charge,
                    count: 2,
                },
                intervening_if: None,
                targets: vec![],
            },
            // CR 602.2: Remove a charge counter: Equipped creature gets +2/+2 until end of turn.
            // Note: Full modal support deferred to PB-37. Implementing mode 1 (+2/+2) only.
            AbilityDefinition::Activated {
                cost: Cost::RemoveCounter { counter: CounterType::Charge, count: 1 },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyBoth(2),
                        filter: EffectFilter::AttachedCreature,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
