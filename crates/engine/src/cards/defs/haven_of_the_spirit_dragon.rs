// Haven of the Spirit Dragon — Land, {T}: Add {C}; restricted any-color mana (TODO); return Dragon/Ugin (TODO)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("haven-of-the-spirit-dragon"),
        name: "Haven of the Spirit Dragon".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast a Dragon creature spell.\n{2}, {T}, Sacrifice this land: Return target Dragon creature card or Ugin planeswalker card from your graveyard to your hand.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // TODO: {T}: Add one mana of any color. Spend this mana only to cast a Dragon creature spell.
            // DSL gap: mana restriction (Dragon creatures only) not expressible.

            // TODO: {2}, {T}, Sacrifice this land: Return target Dragon creature card or Ugin
            // planeswalker card from your graveyard to your hand.
            // DSL gaps: return_from_graveyard with specific-card-name filter;
            // sacrifice-self cost not expressible in Activated.
        ],
        ..Default::default()
    }
}
