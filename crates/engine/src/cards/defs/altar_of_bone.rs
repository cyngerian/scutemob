// Altar of Bone — {G}{W}, Sorcery
// As an additional cost to cast this spell, sacrifice a creature.
// Search your library for a creature card, reveal it, put it into your hand, then shuffle.
//
// NOTE: "As an additional cost to cast this spell, sacrifice a creature" is a spell
// additional cost (CR 601.2b). The DSL has no required_additional_cost field on
// CardDefinition for mandatory spell sacrifice costs — see goblin_grenade.rs.
// The search effect is implemented; the additional-cost sacrifice is omitted per W5 policy.
// TODO: Add AdditionalCost::SacrificeCreature to the Spell DSL to model this.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("altar-of-bone"),
        name: "Altar of Bone".to_string(),
        mana_cost: Some(ManaCost { green: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a creature.\nSearch your library for a creature card, reveal it, put it into your hand, then shuffle.".to_string(),
        abilities: vec![
            // TODO: Stripped per W5 policy — search without mandatory sacrifice cost is wrong
            // game state (free tutor). Needs AdditionalCost::SacrificeCreature on spell DSL.
        ],
        ..Default::default()
    }
}
