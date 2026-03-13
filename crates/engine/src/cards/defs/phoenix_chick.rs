// Phoenix Chick — {R}, Creature — Phoenix 1/1
// "Flying, haste
// This creature can't block.
// Whenever you attack with three or more creatures, you may pay {R}{R}. If you do, return
// this card from your graveyard to the battlefield tapped and attacking with a +1/+1 counter
// on it."
//
// Flying and Haste are implemented.
//
// TODO: DSL gap — "This creature can't block" requires a CantBlock keyword or static
// continuous effect. No such KeywordAbility variant exists in the current DSL.
//
// TODO: DSL gap — the triggered ability requires:
// 1. A trigger that fires when you attack with three or more creatures.
// 2. An optional payment of {R}{R} (modal cost decision).
// 3. Returning this card from graveyard to battlefield tapped and attacking with a counter.
// No such trigger condition or graveyard-to-battlefield-attacking effect exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phoenix-chick"),
        name: "Phoenix Chick".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Phoenix"]),
        oracle_text: "Flying, haste\nThis creature can't block.\nWhenever you attack with three or more creatures, you may pay {R}{R}. If you do, return this card from your graveyard to the battlefield tapped and attacking with a +1/+1 counter on it.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        ..Default::default()
    }
}
