// Parapet Thrasher — {2}{R}{R} Creature — Dragon 4/3
// Flying
// Whenever one or more Dragons you control deal combat damage to an opponent, choose one
// that hasn't been chosen this turn —
// • Destroy target artifact that opponent controls.
// • This creature deals 4 damage to each other opponent.
// • Exile the top card of your library. You may play it this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("parapet-thrasher"),
        name: "Parapet Thrasher".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text:
            "Flying\nWhenever one or more Dragons you control deal combat damage to an opponent, choose one that hasn't been chosen this turn —\n• Destroy target artifact that opponent controls.\n• This creature deals 4 damage to each other opponent.\n• Exile the top card of your library. You may play it this turn."
                .to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever one or more Dragons you control deal combat damage to an opponent,
            // choose one that hasn't been chosen this turn — ..."
            // DSL gaps:
            // 1. TriggerCondition::WhenDealsCombatDamageToPlayer is self-referential (fires only
            //    when the source creature deals damage). There is no WheneverCreatureYouControlDealsCombatDamage
            //    that can filter by subtype (Dragon).
            // 2. The "choose one that hasn't been chosen this turn" modal constraint (tracking
            //    previously chosen modes this turn) is not expressible in the DSL.
            // 3. "Exile top card; you may play it this turn" requires PlayExiledCard which
            //    needs DSL support for play-from-exile with end-of-turn expiry.
        ],
        ..Default::default()
    }
}
