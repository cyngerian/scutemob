// Grenzo, Havoc Raiser — {R}{R}, Legendary Creature — Goblin Rogue 2/2
// Whenever a creature you control deals combat damage to a player, choose one —
// • Goad target creature that player controls.
// • Exile the top card of that player's library. Until end of turn, you may cast
//   that card and spend mana as though it were mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grenzo-havoc-raiser"),
        name: "Grenzo, Havoc Raiser".to_string(),
        mana_cost: Some(ManaCost { red: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Goblin", "Rogue"]),
        oracle_text: "Whenever a creature you control deals combat damage to a player, choose one —\n• Goad target creature that player controls.\n• Exile the top card of that player's library. Until end of turn, you may cast that card and you may spend mana as though it were mana of any color to cast that spell.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: Per-creature combat damage trigger with modal choice.
            // Mode 1 (Goad) works but needs per-creature trigger + target from damaged player.
            // Mode 2 (impulse-draw from opponent) needs PlayExiledCard from opponent's library.
        ],
        ..Default::default()
    }
}
