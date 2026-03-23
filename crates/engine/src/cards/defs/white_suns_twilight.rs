// White Sun's Twilight — {X}{W}{W}, Sorcery
// You gain X life. Create X 1/1 colorless Phyrexian Mite artifact creature tokens with
// toxic 1 and "This token can't block." If X is 5 or more, destroy all other creatures.
//
// TODO: All effects depend on X (the value chosen for {X} in the mana cost). DSL lacks
// EffectAmount::XValue for GainLife and CreateToken count, and Condition::XIsNOrMore for
// the conditional board wipe. Implementing any subset would produce wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("white-suns-twilight"),
        name: "White Sun's Twilight".to_string(),
        mana_cost: Some(ManaCost { white: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "You gain X life. Create X 1/1 colorless Phyrexian Mite artifact creature tokens with toxic 1 and \"This token can't block.\" If X is 5 or more, destroy all other creatures. (Players dealt combat damage by a creature with toxic 1 also get a poison counter.)".to_string(),
        abilities: vec![
            // TODO: X-dependent GainLife, X-dependent CreateToken count, conditional board wipe.
        ],
        ..Default::default()
    }
}
