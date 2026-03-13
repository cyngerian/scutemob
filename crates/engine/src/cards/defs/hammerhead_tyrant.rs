// Hammerhead Tyrant — {4}{U}{U}, Creature — Dragon 6/6
// Flying
// Whenever you cast a spell, return up to one target nonland permanent an opponent controls
// with mana value less than or equal to that spell's mana value to its owner's hand.
// TODO: DSL gap — targeted trigger with a dynamic MV comparison filter (≤ spell's MV)
// is not expressible; TargetFilter has no mana-value-comparison variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hammerhead-tyrant"),
        name: "Hammerhead Tyrant".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever you cast a spell, return up to one target nonland permanent an opponent controls with mana value less than or equal to that spell's mana value to its owner's hand.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: whenever you cast a spell, bounce opponent's nonland permanent with MV ≤ spell's MV
        ],
        ..Default::default()
    }
}
