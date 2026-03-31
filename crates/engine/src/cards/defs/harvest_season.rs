// Harvest Season — {2}{G}, Sorcery
// Search your library for up to X basic land cards, where X is the number of tapped
// creatures you control, put those cards onto the battlefield tapped, then shuffle.
//
// TODO: X = number of tapped creatures you control — EffectAmount::CountTappedCreaturesYouControl
// does not exist. This is a dynamic X count based on a battlefield condition at resolution time.
// No DSL equivalent for variable search count based on tapped creatures. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("harvest-season"),
        name: "Harvest Season".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library for up to X basic land cards, where X is the number of tapped creatures you control, put those cards onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            // TODO: Variable X — EffectAmount::CountTappedCreaturesYouControl does not exist.
            // Cannot express "search for up to X" where X is a dynamic battlefield count.
            // Omitted per W5 policy (partial impl would fix X at a wrong value).
        ],
        ..Default::default()
    }
}
