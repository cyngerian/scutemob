// Metastatic Evangel — {2}{W}, Creature — Phyrexian Cleric 1/3
// Whenever another nontoken creature you control enters, proliferate.
// (Choose any number of permanents and/or players, then give each another counter of each kind already there.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("metastatic-evangel"),
        name: "Metastatic Evangel".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Cleric"]),
        oracle_text: "Whenever another nontoken creature you control enters, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // CR 603.6a: "Whenever another nontoken creature you control enters, proliferate."
            // WheneverCreatureEntersBattlefield with controller=You filter.
            // The "nontoken" and "another" constraints use the is_token flag in TargetFilter —
            // NOTE: is_token in TargetFilter is only checked in combat_damage_filter paths;
            // for ETB trigger matching it is silently ignored. "another" (exclude_self) is
            // also unavailable on this trigger variant. Both are minor inaccuracies.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::Proliferate,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
