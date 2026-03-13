// Dragon Tempest — {1}{R}, Enchantment
// Whenever a creature you control with flying enters, it gains haste until end of turn.
// Whenever a Dragon you control enters, it deals X damage to any target, where X is the
// number of Dragons you control.
//
// TODO: DSL gap — both triggered abilities require complex filtering:
// 1. "Whenever a creature you control with flying enters" — WheneverCreatureEntersBattlefield
//    has a filter but no "has flying" attribute filter.
// 2. "it deals X damage to any target, where X is the number of Dragons you control" —
//    requires a creature-type filter on the ETB trigger and a count-based EffectAmount
//    (EffectAmount::CountCreaturesYouControl with subtype filter) not present in the DSL.
// Both abilities are omitted to avoid incorrect behavior.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragon-tempest"),
        name: "Dragon Tempest".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever a creature you control with flying enters, it gains haste until end of turn.\nWhenever a Dragon you control enters, it deals X damage to any target, where X is the number of Dragons you control.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
