// Flowerfoot Swordmaster — {W}, Creature — Mouse Soldier 1/2; Offspring {2};
// Valiant — Whenever this creature becomes the target of a spell or ability you
// control for the first time each turn, Mice you control get +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flowerfoot-swordmaster"),
        name: "Flowerfoot Swordmaster".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Mouse", "Soldier"]),
        oracle_text: "Offspring {2} (You may pay an additional {2} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.)\nValiant — Whenever this creature becomes the target of a spell or ability you control for the first time each turn, Mice you control get +1/+0 until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Offspring),
            AbilityDefinition::Offspring {
                cost: ManaCost { generic: 2, ..Default::default() },
            },
            // TODO: Valiant — Whenever this creature becomes the target of a spell or ability
            // you control for the first time each turn, Mice you control get +1/+0 until end of turn.
            // Requires TriggerCondition::WhenBecomesTargetOfYourSpellOrAbility (first time per turn)
            // and an Effect that buffs all creatures of subtype Mouse you control. Not yet in DSL.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
    }
}
