// Noxious Revival — {G/P}, Instant
// Put target card from a graveyard on top of its owner's library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("noxious-revival"),
        name: "Noxious Revival".to_string(),
        mana_cost: Some(ManaCost {
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "({G/P} can be paid with either {G} or 2 life.)\nPut target card from a graveyard on top of its owner's library.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to: ZoneTarget::Library {
                    owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                    position: LibraryPosition::Top,
                },
                controller_override: None,
            },
            targets: vec![TargetRequirement::TargetCardInGraveyard(TargetFilter::default())],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
