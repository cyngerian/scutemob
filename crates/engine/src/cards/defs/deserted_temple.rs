// Deserted Temple — Land, {T}: Add {C}. {1},{T}: Untap target land (TODO).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deserted-temple"),
        name: "Deserted Temple".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}, {T}: Untap target land.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {1},{T}: Untap target land — UntapPermanent effect not in DSL
        ],
        ..Default::default()
    }
}
