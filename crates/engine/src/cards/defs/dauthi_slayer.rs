// Dauthi Slayer — {B}{B}, Creature — Dauthi Soldier 2/2; Shadow; attacks each combat if able.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dauthi-slayer"),
        name: "Dauthi Slayer".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Dauthi", "Soldier"]),
        oracle_text:
            "Shadow (This creature can block or be blocked by only creatures with shadow.)\nThis creature attacks each combat if able."
                .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Shadow),
            // CR 508.1d: Attacks each combat if able.
            AbilityDefinition::Keyword(KeywordAbility::MustAttackEachCombat),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
    }
}
