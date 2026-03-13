// Eiganjo, Seat of the Empire — Legendary Land, {T}: Add {W}. Channel ability (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("eiganjo-seat-of-the-empire"),
        name: "Eiganjo, Seat of the Empire".to_string(),
        mana_cost: None,
        types: full_types(&[SuperType::Legendary], &[CardType::Land], &[]),
        oracle_text: "{T}: Add {W}.\nChannel — {2}{W}, Discard this card: It deals 4 damage to target attacking or blocking creature. This ability costs {1} less to activate for each legendary creature you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
            },
            // TODO: Channel — {2}{W}, Discard this card: deals 4 damage to target attacking/blocking creature,
            // cost reduction per legendary creature — Channel keyword + variable cost reduction not in DSL
        ],
        ..Default::default()
    }
}
