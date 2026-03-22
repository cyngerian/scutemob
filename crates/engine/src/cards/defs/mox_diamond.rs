// Mox Diamond — {0} Artifact; ETB replacement: discard land or go to graveyard; {T}: Add any color.
// TODO: ETB replacement effect requiring discard-a-land-or-go-to-graveyard is a DSL gap
// (ReplacementModification has no "discard from hand as cost or put into graveyard" variant).
// The tap ability is fully expressible but would produce wrong game state without the ETB replacement.
// W5 policy: implementing only the mana ability would let Mox Diamond enter for free — wrong.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mox-diamond"),
        name: "Mox Diamond".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "If this artifact would enter, you may discard a land card instead. If you do, put this artifact onto the battlefield. If you don't, put it into its owner's graveyard.\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
