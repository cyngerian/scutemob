// Tymna the Weaver — {1}{W}{B}, Legendary Creature — Human Cleric 2/2
// Lifelink
// At the beginning of each of your postcombat main phases, you may pay X life, where X is
// the number of opponents that were dealt combat damage this turn. If you do, draw X cards.
// Partner (You can have two commanders if both have partner.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tymna-the-weaver"),
        name: "Tymna the Weaver".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Cleric"]),
        oracle_text: "Lifelink\nAt the beginning of each of your postcombat main phases, you may pay X life, where X is the number of opponents that were dealt combat damage this turn. If you do, draw X cards.\nPartner (You can have two commanders if both have partner.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Keyword(KeywordAbility::Partner),
            // TODO: DSL gap — "At the beginning of each of your postcombat main phases" trigger
            // condition does not exist (AtBeginningOfPostcombatMainPhase). Additionally, the
            // life payment and draw count depend on a dynamic count of opponents dealt combat
            // damage this turn, which requires tracking opponent damage state not available in
            // effect targets.
        ],
        ..Default::default()
    }
}
