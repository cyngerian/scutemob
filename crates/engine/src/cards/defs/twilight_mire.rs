// Twilight Mire — filter land, {T}: Add {C}. {B/G}, {T}: Add {B}{B}, {B}{G}, or {G}{G} (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("twilight-mire"),
        name: "Twilight Mire".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{B/G}, {T}: Add {B}{B}, {B}{G}, or {G}{G}.".to_string(),
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
            // TODO: {B/G}, {T}: Add {B}{B}, {B}{G}, or {G}{G}.
            // DSL gap: hybrid mana costs ({B/G}) not expressible in Cost::Mana (ManaCost struct has
            // no hybrid field). Triple-choice mana output also not expressible with current Choose.
        ],
        ..Default::default()
    }
}
