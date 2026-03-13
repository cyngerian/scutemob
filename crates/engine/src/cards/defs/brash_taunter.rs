// Brash Taunter — {4}{R}, Creature — Goblin 1/1
// Indestructible; when dealt damage deals that much to target opponent; activated fight
// TODO: damage-reflection triggered ability and fight activated ability with target opponent
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brash-taunter"),
        name: "Brash Taunter".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Indestructible\nWhenever this creature is dealt damage, it deals that much damage to target opponent.\n{2}{R}, {T}: This creature fights another target creature.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // TODO: WhenDealtDamage trigger redirecting that amount to a target opponent
            // requires a targeted_trigger with a damage-amount variable — not in DSL.
            // TODO: Activated fight ability with {2}{R},{T} cost requires
            // activated_ability_targets (Activated has no targets field) — not in DSL.
        ],
        ..Default::default()
    }
}
