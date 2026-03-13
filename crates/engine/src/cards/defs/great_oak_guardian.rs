// Great Oak Guardian — {5}{G}, Creature — Treefolk 4/5
// Flash, Reach; ETB: creatures target player controls get +2/+2 until end of turn and untap
// TODO: ETB targeted buff to all creatures a player controls (targeted_trigger gap)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("great-oak-guardian"),
        name: "Great Oak Guardian".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Treefolk"]),
        oracle_text: "Flash (You may cast this spell any time you could cast an instant.)\nReach\nWhen this creature enters, creatures target player controls get +2/+2 until end of turn. Untap them.".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            // TODO: ETB trigger granting +2/+2 until end of turn to all creatures a target
            // player controls and untapping them requires a targeted_trigger with
            // ForEach over controlled creatures — not in DSL.
        ],
        ..Default::default()
    }
}
