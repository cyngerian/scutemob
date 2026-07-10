// Scalelord Reckoner — {3}{W}{W}, Creature — Dragon 4/4
// Flying
// Whenever a Dragon you control becomes the target of a spell or ability an opponent controls,
// destroy target nonland permanent that player controls.
// ENGINE-BLOCKED: the effect ("destroy target nonland permanent that player controls") must
// restrict its target to permanents controlled by the *triggering* opponent. No TargetFilter
// can scope to the controller of the triggering spell/ability.
// (The trigger itself is now expressible as TriggerCondition::WhenBecomesTarget
// { scope: Some(Dragon you control), by_opponent: true, include_abilities: true } — PB-AC6.)
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
            // ENGINE-BLOCKED: see file header — the trigger is expressible via PB-AC6's
            // WhenBecomesTarget, but "destroy target nonland permanent THAT PLAYER controls"
            // cannot scope its target to the triggering spell/ability's controller.
        ],
        completeness: Completeness::partial("the effect ('destroy target nonland permanent that player controls') must restrict its target to permanents controlled..."),
        ..Default::default()
    }
}
