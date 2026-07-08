// Tainted Observer — {1}{G}{U}, Creature — Phyrexian Bird 2/3
// Flying, toxic 1; whenever another creature you control enters, you may pay {2}.
// If you do, proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tainted-observer"),
        name: "Tainted Observer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Bird"]),
        oracle_text: "Flying\nToxic 1 (Players dealt combat damage by this creature also get a poison counter.)\nWhenever another creature you control enters, you may pay {2}. If you do, proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Toxic(1)),
            // PB-AC2 (CR 118.12): "Whenever another creature you control enters, you may
            // pay {2}. If you do, proliferate." Beneficial optional-pay wrapper.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: true,
                },
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::Proliferate),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
