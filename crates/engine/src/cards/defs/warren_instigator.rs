// Warren Instigator — {R}{R}, Creature — Goblin Berserker 1/1
// Double strike
// TODO: DSL gap — triggered ability "Whenever this creature deals damage to an opponent, you
//   may put a Goblin creature card from your hand onto the battlefield."
//   (targeted_trigger: WhenDealtCombatDamageTo with hand-to-battlefield placement for a
//   subtype filter not supported in card DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("warren-instigator"),
        name: "Warren Instigator".to_string(),
        mana_cost: Some(ManaCost {
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Berserker"]),
        oracle_text: "Double strike\nWhenever this creature deals damage to an opponent, you may put a Goblin creature card from your hand onto the battlefield.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::DoubleStrike),
        ],
        ..Default::default()
    }
}
