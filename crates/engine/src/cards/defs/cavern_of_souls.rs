// Cavern of Souls — Land
// ETB: choose a creature type. {T}: Add {C}. {T}: Add any color, only for creature
// spells of chosen type (and those spells can't be countered).
// TODO: ETB choice (choose creature type) and conditional mana with
// uncounterability are not expressible in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cavern-of-souls"),
        name: "Cavern of Souls".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, choose a creature type.\n{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type, and that spell can't be countered.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: ETB "choose a creature type" is not expressible in the DSL
            // TODO: {T}: Add one mana of any color — spend only to cast chosen-type creature
            // spells, and those spells can't be countered. DSL gap: no ETB choice mechanism
            // and no mana-spending restriction with anti-counterspell rider.
        ],
        ..Default::default()
    }
}
