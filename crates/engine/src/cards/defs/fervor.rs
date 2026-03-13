// Fervor — {2}{R}, Enchantment
// Creatures you control have haste.
//
// TODO: DSL gap — no EffectFilter::CreaturesYouControl exists for static
// continuous effects. Only EffectFilter::AllCreatures is available, which
// would incorrectly grant haste to opponents' creatures too. Ability omitted
// to avoid incorrect behavior.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fervor"),
        name: "Fervor".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have haste. (They can attack and {T} as soon as they come under your control.)".to_string(),
        abilities: vec![
            // TODO: DSL gap — EffectFilter::CreaturesYouControl not available;
            // AllCreatures would incorrectly grant haste to opponent creatures.
        ],
        ..Default::default()
    }
}
