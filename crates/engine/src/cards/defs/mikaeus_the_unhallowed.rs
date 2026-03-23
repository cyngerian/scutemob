// Mikaeus, the Unhallowed — {3}{B}{B}{B}, Legendary Creature — Zombie Cleric 5/5
// Intimidate
// Whenever a Human deals damage to you, destroy it.
// Other non-Human creatures you control get +1/+1 and have undying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mikaeus-the-unhallowed"),
        name: "Mikaeus, the Unhallowed".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Zombie", "Cleric"],
        ),
        oracle_text: "Intimidate\nWhenever a Human deals damage to you, destroy it.\nOther non-Human creatures you control get +1/+1 and have undying.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // TODO: All abilities stripped per W5 policy — Intimidate alone without
            // the undying grant produces wrong death behavior for all other creatures.
            // DSL gaps:
            // 1. Intimidate — keyword exists but partial impl is wrong game state.
            // 2. "Whenever a Human deals damage to you, destroy it." — trigger on
            //    damage-by-subtype not in DSL.
            // 3. "Other non-Human creatures you control get +1/+1 and have undying." —
            //    subtype exclusion filter + Undying keyword grant not in DSL.
        ],
        ..Default::default()
    }
}
