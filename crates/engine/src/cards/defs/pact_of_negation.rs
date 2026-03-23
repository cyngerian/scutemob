// Pact of Negation — {0}, Instant
// Counter target spell.
// At the beginning of your next upkeep, pay {3}{U}{U}. If you don't, you lose the game.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pact-of-negation"),
        name: "Pact of Negation".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell.\nAt the beginning of your next upkeep, pay {3}{U}{U}. If you don't, you lose the game.".to_string(),
        // TODO: Counter target spell + delayed upkeep trigger "pay {3}{U}{U} or lose the game."
        // {0}-cost unconditional counter without the upkeep payment is game-breaking (KI-2).
        // Stripped per W6 policy until delayed triggers are in DSL.
        abilities: vec![],
        ..Default::default()
    }
}
