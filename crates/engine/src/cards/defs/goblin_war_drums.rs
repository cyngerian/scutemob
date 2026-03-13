// Goblin War Drums — {2}{R}, Enchantment
// Creatures you control have menace.
// TODO: DSL gap — granting a keyword (Menace) to all creatures you control requires
// ApplyContinuousEffect with EffectFilter::CreaturesYouControl; not expressible in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-war-drums"),
        name: "Goblin War Drums".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have menace. (They can't be blocked except by two or more creatures.)".to_string(),
        abilities: vec![
            // TODO: DSL gap — static ability granting Menace to all creatures you control
            // requires ApplyContinuousEffect with EffectFilter::CreaturesYouControl; not supported.
        ],
        ..Default::default()
    }
}
