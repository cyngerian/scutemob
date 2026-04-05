// 74. Wayward Swordtooth — {2}{G}, Creature — Dinosaur 5/5;
// Ascend. You may play an additional land on each of your turns.
// Wayward Swordtooth can't attack or block unless you have the city's blessing.
//
// Ascend keyword and AdditionalLandPlays fully modeled.
//
// TODO: "can't attack or block unless you have the city's blessing" — requires either:
// (a) Condition::Not(Box::new(Condition::HasCitysBlessing)) on an attack/block restriction
//     static, or (b) enforcement in legal_actions.rs checking controller's has_citys_blessing.
// DSL gap: no AbilityDefinition::Static for attack/block restrictions conditioned on
// a game state predicate. PB-18 stax restrictions gate on PermanentFilter, not Condition.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wayward-swordtooth"),
        name: "Wayward Swordtooth".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Dinosaur"]),
        oracle_text: "Ascend (If you control ten or more permanents, you get the city's blessing for the rest of the game.)\nYou may play an additional land on each of your turns.\nWayward Swordtooth can't attack or block unless you have the city's blessing.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ascend),
            // CR 305.2: Static — you may play an additional land on each of your turns.
            AbilityDefinition::AdditionalLandPlays { count: 1 },
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
