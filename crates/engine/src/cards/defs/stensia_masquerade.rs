// Stensia Masquerade — {2}{R}, Enchantment
// Attacking creatures you control have first strike.
// Whenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.
// Madness {2}{R}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stensia-masquerade"),
        name: "Stensia Masquerade".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Attacking creatures you control have first strike.\nWhenever a Vampire you control deals combat damage to a player, put a +1/+1 counter on it.\nMadness {2}{R}".to_string(),
        abilities: vec![
            // TODO: DSL gap — "Attacking creatures you control have first strike."
            // EffectFilter::AttackingCreaturesYouControl does not exist.
            // TODO: DSL gap — "Whenever a Vampire you control deals combat damage to a player"
            // trigger condition not in DSL.
            // TODO: DSL gap — Madness {2}{R}. AltCostKind::Madness does not exist.
        ],
        ..Default::default()
    }
}
