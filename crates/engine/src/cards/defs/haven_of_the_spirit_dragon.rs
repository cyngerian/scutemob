// Haven of the Spirit Dragon — Land
// {T}: Add {C}.
// {T}: Add one mana of any color. Spend this mana only to cast a Dragon creature spell.
// {2}, {T}, Sacrifice: Return target Dragon creature card or Ugin planeswalker card
// from your graveyard to your hand.
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
                targets: vec![],
            },
            // {T}: Add one mana of any color. Spend this mana only to cast a Dragon creature spell.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColorRestricted {
                    player: PlayerTarget::Controller,
                    restriction: ManaRestriction::SubtypeOnly(SubType("Dragon".to_string())),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: {2}, {T}, Sacrifice: Return Dragon/Ugin from graveyard to hand.
            // Blocked on: PB-17 SearchLibrary filter for creature subtype (Dragon) and
            // targeted return-from-graveyard with type union filter (Dragon OR Ugin planeswalker).
        ],
        ..Default::default()
    }
}
