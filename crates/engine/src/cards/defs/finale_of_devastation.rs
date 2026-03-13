// Finale of Devastation — {X}{G}{G}, Sorcery; search library/graveyard for creature,
// if X >= 10 all creatures get +X/+X and haste.
// TODO: DSL gap — search graveyard not expressible (SearchLibrary only covers library);
// conditional X-pump (count_threshold) and mass keyword grant not expressible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("finale-of-devastation"),
        name: "Finale of Devastation".to_string(),
        mana_cost: Some(ManaCost { generic: 0, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library and/or graveyard for a creature card with mana value X or less and put it onto the battlefield. If you search your library this way, shuffle. If X is 10 or more, creatures you control get +X/+X and gain haste until end of turn.".to_string(),
        abilities: vec![],
        // TODO: requires search_graveyard + conditional X-threshold pump + mass haste grant
        ..Default::default()
    }
}
