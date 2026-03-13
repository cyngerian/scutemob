// Urza's Saga — Enchantment Land / Saga with 3 chapter abilities
// TODO: Saga mechanic (lore counters, chapter triggers, sacrifice after III),
// Chapter I: gains "{T}: Add {C}", Chapter II: Construct token creation,
// Chapter III: search library for artifact with mana cost {0} or {1}
// Not expressible in current DSL — requires Saga trigger infrastructure.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("urzas-saga"),
        name: "Urza's Saga".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Enchantment, CardType::Land], &["Urza's", "Saga"]),
        oracle_text: "(As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)\nI — This Saga gains \"{T}: Add {C}.\"\nII — This Saga gains \"{2}, {T}: Create a 0/0 colorless Construct artifact creature token with 'This token gets +1/+1 for each artifact you control.'\"\nIII — Search your library for an artifact card with mana cost {0} or {1}, put it onto the battlefield, then shuffle.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
