// Bident of Thassa — {2}{U}{U}, Legendary Enchantment Artifact
// Whenever a creature you control deals combat damage to a player, you may draw a card.
// {1}{U}, {T}: Creatures your opponents control attack this turn if able.
//
// TODO: "Whenever a creature you control deals combat damage to a player" — this is a
//   per-creature trigger, not WhenDealsCombatDamageToPlayer (which is self only).
//   Needs WheneverCreatureYouControlDealsCombatDamageToPlayer. Not in DSL.
// TODO: "Creatures your opponents control attack this turn if able" — forced attack
//   not expressible in current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bident-of-thassa"),
        name: "Bident of Thassa".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Enchantment, CardType::Artifact],
            &[],
        ),
        oracle_text: "Whenever a creature you control deals combat damage to a player, you may draw a card.\n{1}{U}, {T}: Creatures your opponents control attack this turn if able.".to_string(),
        // TODO: Both abilities require DSL extensions.
        abilities: vec![],
        ..Default::default()
    }
}
