// Unnatural Restoration — {1}{G}, Sorcery
// Return target permanent card from your graveyard to your hand. Proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("unnatural-restoration"),
        name: "Unnatural Restoration".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return target permanent card from your graveyard to your hand. Proliferate. (Choose any number of permanents and/or players, then give each another counter of each kind already there.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // Return target permanent card from your graveyard to hand, then Proliferate.
                // has_card_types uses OR semantics to match any permanent-type card.
                effect: Effect::Sequence(vec![
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                        controller_override: None,
                    },
                    Effect::Proliferate,
                ]),
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_types: vec![
                        CardType::Creature,
                        CardType::Artifact,
                        CardType::Enchantment,
                        CardType::Land,
                        CardType::Planeswalker,
                    ],
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
