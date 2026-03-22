// Oboro, Palace in the Clouds — Legendary Land; {T}: Add {U};
// {1}: Return Oboro to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oboro-palace-in-the-clouds"),
        name: "Oboro, Palace in the Clouds".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "{T}: Add {U}.\n{1}: Return Oboro to its owner's hand.".to_string(),
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
            // {1}: Return Oboro to its owner's hand.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
