// Tandem Lookout — {2}{U}, Creature — Human Scout 2/1
// Soulbond
// As long as Tandem Lookout is paired with another creature, each of those creatures has
// "Whenever this creature deals damage to an opponent, draw a card."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tandem-lookout"),
        name: "Tandem Lookout".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Scout"]),
        oracle_text: "Soulbond (You may pair this creature with another unpaired creature when either enters. They remain paired for as long as you control both of them.)\nAs long as Tandem Lookout is paired with another creature, each of those creatures has \"Whenever this creature deals damage to an opponent, draw a card.\"".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Soulbond),
            // TODO: Soulbond grant of "deals damage to opponent → draw" not in DSL.
            //   SoulbondGrant struct lacks triggered ability grants.
        ],
        ..Default::default()
    }
}
