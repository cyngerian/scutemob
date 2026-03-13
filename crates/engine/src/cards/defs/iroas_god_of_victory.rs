// Iroas, God of Victory — {2}{R}{W}, Legendary Enchantment Creature — God 7/4
// "Indestructible
// As long as your devotion to red and white is less than seven, Iroas isn't a creature.
// Creatures you control have menace.
// Prevent all damage that would be dealt to attacking creatures you control."
//
// Indestructible is implemented.
//
// TODO: DSL gap — devotion-based "isn't a creature" requires a conditional type-removal
// continuous effect (Layer 4) parameterized on devotion count — no such effect in DSL.
//
// TODO: DSL gap — "Creatures you control have menace" is a continuous keyword-grant effect
// (Layer 6) applying to all creatures you control — no EffectFilter for "all creatures you
// control" in a static continuous ability.
//
// TODO: DSL gap — "Prevent all damage that would be dealt to attacking creatures you control"
// is a blanket prevention replacement effect scoped to attacking creatures — no such
// replacement pattern exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("iroas-god-of-victory"),
        name: "Iroas, God of Victory".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Enchantment, CardType::Creature],
            &["God"],
        ),
        oracle_text: "Indestructible\nAs long as your devotion to red and white is less than seven, Iroas isn't a creature.\nCreatures you control have menace.\nPrevent all damage that would be dealt to attacking creatures you control.".to_string(),
        power: Some(7),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
        ],
        ..Default::default()
    }
}
