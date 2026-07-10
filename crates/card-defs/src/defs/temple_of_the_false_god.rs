// Temple of the False God — Land
// {T}: Add {C}{C}. Activate only if you control five or more lands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("temple-of-the-false-god"),
        name: "Temple of the False God".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}{C}. Activate only if you control five or more lands.".to_string(),
        abilities: vec![
            // {T}: Add {C}{C}. Activate only if you control five or more lands.
            // "five or more lands" = four or more OTHER lands (since Temple itself is a land).
            // ControlAtLeastNOtherLands(4) excludes the source from the count.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 2),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: Some(Condition::ControlAtLeastNOtherLands(4)),
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
