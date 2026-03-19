// 74. Wayward Swordtooth — {2}{G}, Creature — Dinosaur 5/5;
// Ascend. You may play an additional land on each of your turns.
// Wayward Swordtooth can't attack or block unless you have the city's blessing.
//
// Ascend keyword fully modeled.
//
// TODO: "You may play an additional land on each of your turns" — requires an
// AdditionalLandPlay static effect that increments the per-turn land play allowance.
// DSL gap: no AbilityDefinition or LayerModification for extra land plays.
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
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
