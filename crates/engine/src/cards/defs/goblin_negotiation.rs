// Goblin Negotiation — {X}{R}{R}, Sorcery
// Goblin Negotiation deals X damage to target creature. Create a number of 1/1 red Goblin
// creature tokens equal to the amount of excess damage dealt to that creature this way.
//
// TODO: Both effects depend on X and excess damage calculation:
// - DealDamage with EffectAmount::XValue not in DSL
// - "Create tokens equal to excess damage dealt" requires tracking toughness vs damage dealt,
//   which is a complex runtime calculation not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-negotiation"),
        name: "Goblin Negotiation".to_string(),
        mana_cost: Some(ManaCost { red: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Goblin Negotiation deals X damage to target creature. Create a number of 1/1 red Goblin creature tokens equal to the amount of excess damage dealt to that creature this way.".to_string(),
        abilities: vec![
            // TODO: X-dependent DealDamage and excess-damage token creation — see comment above.
        ],
        ..Default::default()
    }
}
