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
            // WheneverCreatureEntersBattlefield with controller=You filter +
            // exclude_self: true (PB-XS-E, CR 109.1 / 603.2). The "nontoken" qualifier
            // uses is_token in TargetFilter — NOTE: is_token in TargetFilter is only
            // checked in combat_damage_filter paths; for ETB trigger matching it is
            // silently ignored. Minor inaccuracy: a token creature ETB would still fire
            // this trigger today (until ETBTriggerFilter gains a token-only/nontoken-only
            // axis). Tracked elsewhere.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: true,
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
