// Unearth — {B}, Sorcery
// Return target creature card with mana value 3 or less from your graveyard to the battlefield.
// Cycling {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("unearth"),
        name: "Unearth".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return target creature card with mana value 3 or less from your graveyard to the battlefield.\nCycling {2} ({2}, Discard this card: Draw a card.)".to_string(),
        abilities: vec![
            // TODO: TargetCardInYourGraveyard lacks mana value filter (<=3).
            //   Using unfiltered graveyard targeting as approximation.
            AbilityDefinition::Spell {
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield {
                        tapped: false,
                    },
                    controller_override: None,
                },
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
            AbilityDefinition::Keyword(KeywordAbility::Cycling),
            AbilityDefinition::Cycling {
                cost: ManaCost { generic: 2, ..Default::default() },
            },
        ],
        ..Default::default()
    }
}
