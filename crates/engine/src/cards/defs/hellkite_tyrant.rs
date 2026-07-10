// Hellkite Tyrant — {4}{R}{R}, Creature — Dragon 6/5
// Flying, trample
// Whenever this creature deals combat damage to a player, gain control of all artifacts that player controls.
// At the beginning of your upkeep, if you control twenty or more artifacts, you win the game.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hellkite-tyrant"),
        name: "Hellkite Tyrant".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying, trample\nWhenever this creature deals combat damage to a player, gain control of all artifacts that player controls.\nAt the beginning of your upkeep, if you control twenty or more artifacts, you win the game.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: ENGINE-BLOCKED — "Whenever this creature deals combat damage to a player,
            // gain control of all artifacts that player controls." DSL gap: no "gain control of
            // all permanents of type" effect targeting the damaged player. PARTIAL per PB-AC8 roster.
            // "At the beginning of your upkeep, if you control twenty or more artifacts, you win
            // the game." — now expressible via Effect::WinGame + Condition::YouControlNOrMoreWithFilter
            // (PB-AC8). CR 603.4 intervening-if re-checked at resolution.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::WinGame,
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 20,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Artifact),
                        ..Default::default()
                    },
                }),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial("ENGINE-BLOCKED — 'Whenever this creature deals combat damage to a player, gain control of all artifacts that player..."),
        ..Default::default()
    }
}
