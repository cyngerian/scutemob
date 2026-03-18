// Hall of Heliod's Generosity — Legendary Land
// {T}: Add {C}.
// {1}{W}, {T}: Put target enchantment card from your graveyard on top of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hall-of-heliods-generosity"),
        name: "Hall of Heliod's Generosity".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}{W}, {T}: Put target enchantment card from your graveyard on top of your library.".to_string(),
        abilities: vec![
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // {1}{W}, {T}: Put target enchantment card from your GY on top of library.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, white: 1, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Library {
                        owner: PlayerTarget::Controller,
                        position: LibraryPosition::Top,
                    },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Enchantment),
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}
