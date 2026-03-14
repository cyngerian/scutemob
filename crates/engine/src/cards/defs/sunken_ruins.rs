// Sunken Ruins — Filter land, {T}: Add {C}. {U/B},{T}: Add {U}{U}, {U}{B}, or {B}{B} (TODO: filter).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sunken-ruins"),
        name: "Sunken Ruins".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{U/B}, {T}: Add {U}{U}, {U}{B}, or {B}{B}.".to_string(),
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
            // TODO: {U/B},{T}: Add {U}{U}, {U}{B}, or {B}{B} — hybrid cost and triple-choice filter ability not in DSL
        ],
        ..Default::default()
    }
}
