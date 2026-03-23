// Braids, Arisen Nightmare — {1}{B}{B}, Legendary Creature — Nightmare 3/3
// At the beginning of your end step, you may sacrifice an artifact, creature,
// enchantment, land, or planeswalker. If you do, each opponent may sacrifice a
// permanent that shares a card type with it. For each opponent who doesn't, that
// player loses 2 life and you draw a card.
//
// TODO: Complex end-step trigger with sacrifice choice, type-matching opponent sacrifice,
//   and conditional draw per opponent who doesn't sacrifice. Requires multiple player
//   choices and type-matching. Too complex for current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("braids-arisen-nightmare"),
        name: "Braids, Arisen Nightmare".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Nightmare"],
        ),
        oracle_text: "At the beginning of your end step, you may sacrifice an artifact, creature, enchantment, land, or planeswalker. If you do, each opponent may sacrifice a permanent of their choice that shares a card type with it. For each opponent who doesn't, that player loses 2 life and you draw a card.".to_string(),
        power: Some(3),
        toughness: Some(3),
        // TODO: Full ability requires sacrifice-choice, type-matching, per-opponent
        //   decision, conditional life loss + draw. Not expressible in current DSL.
        abilities: vec![],
        ..Default::default()
    }
}
