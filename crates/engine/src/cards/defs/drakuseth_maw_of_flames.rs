// Drakuseth, Maw of Flames — {4}{R}{R}{R}, Legendary Creature — Dragon 7/7
// Flying
// Whenever Drakuseth attacks, it deals 4 damage to any target and 3 damage to each
// of up to two other targets.
//
// NOTE: "3 damage to each of up to two other targets" requires declaring up to two
// additional targets on WhenAttacks trigger resolution with separate damage applications.
// The DSL DeclaredTarget supports one index per target but the "up to two other" pattern
// requires optional multi-target selection not expressible in the current DSL. Per W5,
// a partial implementation (only the 4-damage portion) would produce wrong game state.
// Use TODO for the full trigger per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("drakuseth-maw-of-flames"),
        name: "Drakuseth, Maw of Flames".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever Drakuseth attacks, it deals 4 damage to any target and 3 damage to each of up to two other targets.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever Drakuseth attacks, it deals 4 damage to any target and 3 damage to
            // each of up to two other targets." — requires multi-target declaration on a triggered
            // ability (one required target + up to two optional other targets). The DSL supports
            // a single DeclaredTarget list but has no "up to two other" pattern for triggered
            // abilities. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
