// Black Market Connections — {2}{B} Enchantment
// At the beginning of your first main phase, choose one or more —
// • Sell Contraband — Create a Treasure token. You lose 1 life.
// • Buy Information — Draw a card. You lose 2 life.
// • Hire a Mercenary — Create a 3/2 colorless Shapeshifter creature token with changeling.
//   You lose 3 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("black-market-connections"),
        name: "Black Market Connections".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your first main phase, choose one or more —\n• Sell Contraband — Create a Treasure token. You lose 1 life.\n• Buy Information — Draw a card. You lose 2 life.\n• Hire a Mercenary — Create a 3/2 colorless Shapeshifter creature token with changeling. You lose 3 life.".to_string(),
        abilities: vec![
            // ENGINE-BLOCKED: triggered abilities carry a ModeSelection and resolve it
            // (CR 700.2b), but the mode CHOICE is not player-declared — `abilities.rs` hardcodes
            // `stack_obj.modes_chosen = vec![0]` when queuing a modal trigger. Authoring this
            // card would silently always pick "Sell Contraband" and never offer the other two,
            // and "choose one or more" (max_modes = 3) could never select a second mode.
            // Needs player-declared mode choice on triggered abilities.
            // (The "at the beginning of your first main phase" trigger itself is now
            // available as TriggerCondition::AtBeginningOfFirstMainPhase — PB-AC6.)
        ],
        completeness: Completeness::partial("triggered abilities carry a ModeSelection and resolve it (CR 700.2b), but the mode CHOICE is not player-declared..."),
        ..Default::default()
    }
}
