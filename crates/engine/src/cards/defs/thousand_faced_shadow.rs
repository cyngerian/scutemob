// Thousand-Faced Shadow
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thousand-faced-shadow"),
        name: "Thousand-Faced Shadow".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {2}{U}{U} ({2}{U}{U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nFlying\nWhen this creature enters from your hand, if it's attacking, create a token that's a copy of another target attacking creature. The token enters tapped and attacking.".to_string(),
        abilities: vec![
            // TODO: Keyword — Ninjutsu {2}{U}{U} ({2}{U}{U}, Return an unblocked attacker you control to hand:
            // TODO: Keyword — Flying
            // TODO: Triggered — When this creature enters from your hand, if it's attacking, create a token that
        ],
        power: Some(1),
        toughness: Some(1),
        ..Default::default()
    }
}
