// Balefire Dragon — {5}{R}{R}, Creature — Dragon 6/6
// Flying
// Whenever this creature deals combat damage to a player, it deals that much damage to
// each creature that player controls.
//
// TODO: "each creature that player controls" requires ForEach over DamagedPlayer's creatures,
//   which needs TargetController::DamagedPlayer support in ForEach filters. Deferred to PB-37.
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
            // TODO: "it deals that much damage to each creature that player controls" —
            //   ForEach over DamagedPlayer's creatures not in DSL. Deferred to PB-37.
        ],
        ..Default::default()
    }
}
