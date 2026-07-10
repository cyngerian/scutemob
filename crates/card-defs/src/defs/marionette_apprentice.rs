// Marionette Apprentice — {1}{B}, Creature — Human Artificer 1/2
// Fabricate 1
// Whenever another creature or artifact you control is put into a graveyard from the
// battlefield, each opponent loses 1 life.
//
// PARTIAL: Fabricate 1 keyword is implemented.
// ENGINE-BLOCKED: "Whenever another creature or artifact you control dies" — there is no
// TriggerCondition covering both creatures AND artifacts dying from the battlefield.
// WheneverCreatureDies only covers creatures; no artifact-dies or permanent-dies trigger
// variant exists. A creature-only trigger would silently miss artifact deaths = wrong game
// state. Trigger omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("marionette-apprentice"),
        name: "Marionette Apprentice".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Artificer"]),
        oracle_text: "Fabricate 1 (When this creature enters, put a +1/+1 counter on it or create a 1/1 colorless Servo artifact creature token.)\nWhenever another creature or artifact you control is put into a graveyard from the battlefield, each opponent loses 1 life.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Fabricate(1)),
            // ENGINE-BLOCKED: "Whenever another creature or artifact you control is put into
            // a graveyard from the battlefield, each opponent loses 1 life."
            // Requires a TriggerCondition for artifact OR creature dying that is currently
            // absent from the DSL. WheneverCreatureDies covers only creatures; applying it
            // here would miss artifact deaths and produce incorrect game state.
        ],
        completeness: Completeness::partial("'Whenever another creature or artifact you control dies' — there is no TriggerCondition covering both creatures AND..."),
        ..Default::default()
    }
}
