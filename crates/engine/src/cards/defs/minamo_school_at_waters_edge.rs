// Minamo, School at Water's Edge — Legendary Land
// {T}: Add {U}. {U},{T}: Untap target legendary permanent (untap ability not expressible).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("minamo-school-at-waters-edge"),
        name: "Minamo, School at Water's Edge".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {U}.\n{U}, {T}: Untap target legendary permanent.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
            // TODO: {U},{T}: Untap target legendary permanent — tap-plus-pay-mana
            // cost with untap-permanent effect is not expressible in the DSL
            // (no Effect::UntapPermanent or combined mana+tap Cost variant)
        ],
        ..Default::default()
    }
}
