// Fauna Shaman — {1}{G} Creature — Elf Shaman 2/2
// {G}, {T}, Discard a creature card: Search your library for a creature card, reveal it, put it into your hand, then shuffle.
//
// DSL gap: "discard a creature card" as part of an activated ability cost is not in DSL
//   (Cost enum has no DiscardCardWithType variant).
// W5 policy: cannot faithfully express discard-type-as-cost — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fauna-shaman"),
        name: "Fauna Shaman".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text: "{G}, {T}, Discard a creature card: Search your library for a creature card, reveal it, put it into your hand, then shuffle.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: {G}, {T}, Discard a creature card: search for creature card, put into hand
            //   (Cost enum lacks DiscardCardWithType(TargetFilter) variant)
        ],
        ..Default::default()
    }
}
