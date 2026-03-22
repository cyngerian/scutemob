// Scalelord Reckoner — {3}{W}{W}, Creature — Dragon 4/4
// Flying
// Whenever a Dragon you control becomes the target of a spell or ability an opponent controls,
// destroy target nonland permanent that player controls.
// TODO: DSL gap — "whenever a Dragon you control becomes the target of a spell or ability an
// opponent controls" requires TriggerCondition::WhenPermanentYouControlBecomesTarget with a
// subtype filter (Dragon) and opponent-controller restriction; no such trigger exists in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scalelord-reckoner"),
        name: "Scalelord Reckoner".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever a Dragon you control becomes the target of a spell or ability an opponent controls, destroy target nonland permanent that player controls.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever a Dragon you control becomes the target of a spell or ability an
            // opponent controls, destroy target nonland permanent that player controls."
            // DSL gap: no TriggerCondition for "permanent you control becomes target of opponent
            // spell/ability". Requires trigger + subtype filter + opponent-scoped permanent target.
        ],
        ..Default::default()
    }
}
