// Marisi, Breaker of the Coil — {1}{R}{G}{W}, Legendary Creature — Cat Warrior 5/4
// Your opponents can't cast spells during combat.
// Whenever a creature you control deals combat damage to a player, goad each creature
// that player controls.
//
// TODO: "Your opponents can't cast spells during combat" — phase-scoped CantCast not in DSL.
// TODO: "goad each creature that player controls" — ForEach over DamagedPlayer's creatures
//   not in DSL. Deferred to PB-37.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("marisi-breaker-of-the-coil"),
        name: "Marisi, Breaker of the Coil".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, green: 1, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Cat", "Warrior"]),
        oracle_text: "Your opponents can't cast spells during combat.\nWhenever a creature you control deals combat damage to a player, goad each creature that player controls.".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            // TODO: "Your opponents can't cast spells during combat" — phase-scoped CantCast not in DSL.
            // TODO: "goad each creature that player controls" — ForEach over DamagedPlayer's
            //   creatures requires TargetController::DamagedPlayer support. Deferred to PB-37.
        ],
        ..Default::default()
    }
}
