// Access Denied — {3}{U}{U}, Instant
// Counter target spell. Create X 1/1 colorless Thopter artifact creature tokens with
// flying, where X is that spell's mana value.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("access-denied"),
        name: "Access Denied".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. Create X 1/1 colorless Thopter artifact creature tokens with flying, where X is that spell's mana value.".to_string(),
        // TODO: Counter target spell + create X 1/1 Thopter tokens where X = MV.
        // Counter without tokens removes the card's primary payoff (KI-2).
        // Stripped per W6 policy until MV-tracking + variable token count is in DSL.
        abilities: vec![],
        ..Default::default()
    }
}
