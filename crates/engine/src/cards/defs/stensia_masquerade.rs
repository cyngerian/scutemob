// Stensia Masquerade — {2}{R}, Enchantment
// Attacking creatures you control have first strike.
// Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.
// Madness {2}{R}
//
// TODO: Madness {2}{R} — AltCostKind::Madness not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stensia-masquerade"),
        name: "Stensia Masquerade".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking creatures you control have first strike.\nWhenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.\nMadness {2}{R}".to_string(),
        abilities: vec![
            // CR 613.1f / CR 611.3a: "Attacking creatures you control have first strike."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
                    filter: EffectFilter::AttackingCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 510.3a: "Whenever a Vampire you control deals combat damage to a player,
            // put a +1/+1 counter on it." — per-creature trigger with Vampire filter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Vampire".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::TriggeringCreature,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
