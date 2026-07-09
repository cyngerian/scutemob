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
            // ENGINE-BLOCKED: Valiant needs "targeted by a spell or ability YOU control".
            // PB-AC6's WhenBecomesTarget.by_opponent is a bool — false means "any controller",
            // true means "opponent only". There is no you-control-only variant, so authoring
            // this would also fire on opponents' spells (wrong game state). Needs a
            // three-way controller scope on WhenBecomesTarget.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
    }
}
