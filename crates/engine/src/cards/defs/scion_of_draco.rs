// Scion of Draco — {12}, Artifact Creature — Dragon 4/4
// Domain — This spell costs {2} less to cast for each basic land type among lands you control.
// Flying
// TODO: DSL gap — static ability: "Each creature you control has vigilance if it's white,
//   hexproof if it's blue, lifelink if it's black, first strike if it's red, and trample if
//   it's green." (color-conditional keyword grant not supported in card DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scion-of-draco"),
        name: "Scion of Draco".to_string(),
        mana_cost: Some(ManaCost {
            generic: 12,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Dragon"]),
        oracle_text: "Domain — This spell costs {2} less to cast for each basic land type among lands you control.\nFlying\nEach creature you control has vigilance if it's white, hexproof if it's blue, lifelink if it's black, first strike if it's red, and trample if it's green.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: color-conditional keyword grant to creatures you control
        ],
        self_cost_reduction: Some(SelfCostReduction::BasicLandTypes { per: 2 }),
        ..Default::default()
    }
}
