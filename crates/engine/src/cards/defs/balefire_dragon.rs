// Balefire Dragon — {5}{R}{R}, Creature — Dragon 6/6
// Flying
// Whenever this creature deals combat damage to a player, it deals that much damage to
// each creature that player controls.
//
// Flying is implemented.
//
// TODO: DSL gap — "Whenever this creature deals combat damage to a player, it deals that
// much damage to each creature that player controls" requires:
// 1. TriggerCondition for combat damage dealt by this specific creature to a player.
// 2. EffectAmount equal to the damage dealt (no "amount equal to combat damage dealt" variant).
// 3. Effect targeting each creature the damaged player controls.
// This ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("balefire-dragon"),
        name: "Balefire Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 5, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhenever this creature deals combat damage to a player, it deals that much damage to each creature that player controls.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        ..Default::default()
    }
}
