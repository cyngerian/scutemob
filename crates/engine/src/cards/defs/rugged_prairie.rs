// Rugged Prairie — filter land, {T}: Add {C}. {R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W} (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rugged-prairie"),
        name: "Rugged Prairie".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {R/W}, {T}: Add {R}{R}, {R}{W}, or {W}{W}.
            // DSL gap: hybrid mana costs ({R/W}) not expressible in Cost::Mana (ManaCost struct has
            // no hybrid field). Triple-choice mana output also not expressible with current Choose.
        ],
        ..Default::default()
    }
}
