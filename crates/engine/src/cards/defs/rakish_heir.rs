// Rakish Heir — {2}{R}, Creature — Vampire 2/2
// Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rakish-heir"),
        name: "Rakish Heir".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 510.3a: "Whenever a Vampire you control deals combat damage to a player,
            // put a +1/+1 counter on it." — per-creature trigger with Vampire subtype filter.
            // "it" = the dealing creature (TriggeringCreature).
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
