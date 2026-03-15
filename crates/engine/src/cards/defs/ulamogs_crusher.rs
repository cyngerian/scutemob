// 62. Ulamog's Crusher — {8}, Creature — Eldrazi 8/8.
// Annihilator 2 (CR 702.86a): whenever this creature attacks, the defending
// player sacrifices two permanents. Engine support: builder.rs registers the
// WhenAttacks triggered ability from KeywordAbility::Annihilator(n).
// "This creature attacks each combat if able." — enforced via MustAttackEachCombat keyword.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ulamogs-crusher"),
        name: "Ulamog's Crusher".to_string(),
        mana_cost: Some(ManaCost { generic: 8, ..Default::default() }),
        types: creature_types(&["Eldrazi"]),
        oracle_text: "Annihilator 2 (Whenever this creature attacks, defending player sacrifices two permanents.)\nThis creature attacks each combat if able.".to_string(),
        power: Some(8),
        toughness: Some(8),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Annihilator(2)),
            // CR 508.1d: Attacks each combat if able.
            AbilityDefinition::Keyword(KeywordAbility::MustAttackEachCombat),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
