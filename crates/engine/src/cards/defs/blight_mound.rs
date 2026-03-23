// Blight Mound — {2}{B}, Enchantment
// Attacking Pests you control get +1/+0 and have menace.
// Whenever a nontoken creature you control dies, create a 1/1 black and green Pest creature
// token with "When this token dies, you gain 1 life."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blight-mound"),
        name: "Blight Mound".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking Pests you control get +1/+0 and have menace.\nWhenever a nontoken creature you control dies, create a 1/1 black and green Pest creature token with \"When this token dies, you gain 1 life.\"".to_string(),
        abilities: vec![
            // TODO: Static lord effect — "Attacking Pests you control get +1/+0 and have menace"
            // requires EffectFilter::AttackingCreaturesYouControlWithSubtype, not in DSL.

            // TODO: "Whenever a nontoken creature you control dies" — WhenDies trigger with
            // nontoken + controller filter not expressible. Token has nested triggered ability
            // ("When this token dies, you gain 1 life") which also cannot be expressed in DSL.
        ],
        ..Default::default()
    }
}
