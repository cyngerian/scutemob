// Metastatic Evangel — {1}{W}, Creature — Phyrexian Human Cleric 3/1
// Whenever another nontoken creature you control enters, proliferate.
// (Choose any number of permanents and/or players, then give each another counter of each kind already there.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("metastatic-evangel"),
        name: "Metastatic Evangel".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: creature_types(&["Phyrexian", "Human", "Cleric"]),
        oracle_text: "Whenever another nontoken creature you control enters, proliferate. (Choose \
                      any number of permanents and/or players, then give each another counter of \
                      each kind already there.)"
            .to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            // CR 603.6a: "Whenever another nontoken creature you control enters, proliferate."
            // WheneverCreatureEntersBattlefield with controller=You filter +
            // exclude_self: true (PB-XS-E, CR 109.1 / 603.2). The "nontoken" qualifier is
            // honoured via `is_nontoken` on the creature-ETB path: `triggering_creature_filter`
            // forwards the full TargetFilter (PB-AC0), and `rules/abilities.rs`'s
            // WheneverCreatureEntersBattlefield arm checks `creature_filter.is_nontoken` against
            // the entering GameObject before firing (CR 111.1 — is_token/is_nontoken are runtime
            // GameObject properties, not Characteristics, so they can't live in matches_filter()).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        is_nontoken: true,
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
