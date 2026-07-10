// Galadhrim Ambush — {3}{G}, Instant
// Create X 1/1 green Elf Warrior creature tokens, where X is the number of attacking creatures.
// Prevent all combat damage that would be dealt this turn by non-Elf creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("galadhrim-ambush"),
        name: "Galadhrim Ambush".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Create X 1/1 green Elf Warrior creature tokens, where X is the number of attacking creatures.\nPrevent all combat damage that would be dealt this turn by non-Elf creatures.".to_string(),
        abilities: vec![
            // PB-AC3: "X = number of attacking creatures" is now expressible via
            // EffectAmount::AttackingCreatureCount — NOT the remaining blocker.
            // TODO: "Prevent all combat damage that would be dealt this turn by non-Elf
            // creatures" — a filtered (subtype-exclusion) damage-prevention shield is still
            // a DSL gap (no prevention-effect primitive accepts a TargetFilter). Under W6
            // no-partials policy this two-clause instant stays fully blocked rather than
            // authoring only the token-creation half (which would silently drop the
            // prevention clause the card's text promises).
        ],
        completeness: Completeness::partial("'Prevent all combat damage that would be dealt this turn by non-Elf creatures' — a filtered (subtype-exclusion)..."),
        ..Default::default()
    }
}
