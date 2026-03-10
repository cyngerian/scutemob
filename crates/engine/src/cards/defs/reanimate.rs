// Reanimate — {B} Sorcery; put target creature card from a graveyard onto
// the battlefield under your control. You lose life equal to its mana value.
// TODO: DSL gap — "target creature card in a graveyard" targeting and
// "lose life equal to its mana value" dynamic life loss are not expressible.
// Empty abilities per W5 policy (no-op placeholder would make it castable).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reanimate"),
        name: "Reanimate".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Put target creature card from a graveyard onto the battlefield under your control. You lose life equal to its mana value.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
