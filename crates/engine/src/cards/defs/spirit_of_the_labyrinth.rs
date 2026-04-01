// Spirit of the Labyrinth — {1}{W}, Enchantment Creature — Spirit 3/1
// Each player can't draw more than one card each turn.
//
// TODO: "Each player can't draw more than one card each turn" — a static restriction on
//   draw count per turn. No GameRestriction::CantDrawMoreThanOncePerTurn or
//   ContinuousEffectDef modification for capping draws exists in DSL.
//   DSL gap: need a LayerModification::RestrictDrawsPerTurn (or similar) for static
//   enchantment/creature abilities that cap card drawing. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spirit-of-the-labyrinth"),
        name: "Spirit of the Labyrinth".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Spirit"]),
        oracle_text: "Each player can't draw more than one card each turn.".to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            // TODO: Static ability "each player can't draw more than one card each turn."
            //   DSL gap: no LayerModification or GameRestriction to cap draws per turn.
            //   Requires new engine primitive. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
