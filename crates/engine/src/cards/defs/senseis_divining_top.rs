// Sensei's Divining Top — {1}, Artifact
// {1}: Look at the top three cards of your library, then put them back in any order.
// {T}: Draw a card, then put this artifact on top of its owner's library.
//
// TODO: {1} ability — "look at top 3, put back in any order" requires a rearrange/order
// top-of-library effect that does not exist in the DSL. Omitted per W5 policy.
// TODO: {T} ability — "draw a card, then put this artifact on top of its owner's library"
// requires Effect::PutSelfOnTopOfLibrary which does not exist in the DSL. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("senseis-divining-top"),
        name: "Sensei's Divining Top".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{1}: Look at the top three cards of your library, then put them back in any order.\n{T}: Draw a card, then put this artifact on top of its owner's library.".to_string(),
        abilities: vec![
            // TODO: {1}: Look at top 3 and rearrange — no rearrange-top-of-library effect in DSL.
            // TODO: {T}: Draw a card then put self on top of library — no PutSelfOnTopOfLibrary effect.
        ],
        ..Default::default()
    }
}
