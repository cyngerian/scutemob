// Graven Cairns — Filter land, {T}: Add {C}. {B/R},{T}: Add {B}{B}, {B}{R}, or {R}{R} (TODO: filter).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("graven-cairns"),
        name: "Graven Cairns".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{B/R}, {T}: Add {B}{B}, {B}{R}, or {R}{R}.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {B/R},{T}: Add {B}{B}, {B}{R}, or {R}{R} — hybrid cost and triple-choice filter ability not in DSL
        ],
        ..Default::default()
    }
}
