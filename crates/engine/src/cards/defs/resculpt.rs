// Resculpt — {1}{U}, Instant
// Exile target artifact or creature. Its controller creates a 4/4 blue and red Elemental
// creature token.
// TODO: Token should go to the controller of the exiled permanent, not the spell caster.
//   Effect::CreateToken always creates for the spell controller. In multiplayer this produces
//   wrong game state when targeting an opponent's permanent. Per W5 policy, leaving abilities
//   empty rather than implementing with wrong token recipient.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("resculpt"),
        name: "Resculpt".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target artifact or creature. Its controller creates a 4/4 blue and red Elemental creature token.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
