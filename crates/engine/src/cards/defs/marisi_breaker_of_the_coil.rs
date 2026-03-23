// Marisi, Breaker of the Coil — {1}{R}{G}{W}, Legendary Creature — Cat Warrior 5/4
// Your opponents can't cast spells during combat.
// Whenever a creature you control deals combat damage to a player, goad each creature
// that player controls.
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
            // TODO: DSL gap — "Your opponents can't cast spells during combat" is a static
            // ability creating a casting restriction (CantCast) scoped to the combat phase
            // for opponent players. No such phase-scoped CantCast restriction exists in the DSL.
            // TODO: DSL gap — "Whenever a creature you control deals combat damage to a player,
            // goad each creature that player controls" requires a combat-damage trigger on any
            // creature you control plus goading all creatures of the damaged player. No
            // combat-damage trigger for creatures-you-control exists in the DSL.
        ],
        ..Default::default()
    }
}
