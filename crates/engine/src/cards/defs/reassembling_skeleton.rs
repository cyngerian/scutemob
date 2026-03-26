// Reassembling Skeleton — {1}{B}, Creature — Skeleton Warrior 1/1.
// "{1}{B}: Return Reassembling Skeleton from your graveyard to the
// battlefield tapped."
// TODO: DSL gap — activated ability from graveyard zone not expressible.
// Only the creature body is defined.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reassembling-skeleton"),
        name: "Reassembling Skeleton".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Skeleton", "Warrior"]),
        oracle_text: "{1}{B}: Return Reassembling Skeleton from your graveyard to the battlefield tapped.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        // TODO: activated ability from graveyard
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
