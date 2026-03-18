// Haven of the Spirit Dragon — Land
// {T}: Add {C}.
// {T}: Add one mana of any color. Spend this mana only to cast a Dragon creature spell.
// {2}, {T}, Sacrifice: Return target Dragon creature card or Ugin planeswalker card
//   from your graveyard to your hand.
// Third ability uses graveyard targeting (PB-10) with Dragon subtype filter.
// TODO: "or Ugin planeswalker card" part of the target filter (name + type union)
//   is not expressible. Currently only targets Dragon creature cards.
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
                    restriction: ManaRestriction::CreatureWithSubtype(SubType("Dragon".to_string())),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // {2}, {T}, Sacrifice: Return target Dragon creature card from graveyard to hand.
            // TODO: Also targets "Ugin planeswalker card" (name + type union not expressible)
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 2,
                        ..Default::default()
                    }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    has_subtype: Some(SubType("Dragon".to_string())),
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}
