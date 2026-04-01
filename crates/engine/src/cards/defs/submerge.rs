// Submerge — {4}{U}, Instant
// If an opponent controls a Forest and you control an Island, you may cast this
// spell without paying its mana cost.
// Put target creature on top of its owner's library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("submerge"),
        name: "Submerge".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If an opponent controls a Forest and you control an Island, you may cast this spell without paying its mana cost.\nPut target creature on top of its owner's library.".to_string(),
        abilities: vec![
            // TODO: Conditional free-cast "if opponent controls a Forest and you control
            // an Island" — no AltCostKind for land-conditional free cast. The spell effect
            // (put creature on top of library) works correctly.
            AbilityDefinition::Spell {
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Library {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })),
                        position: LibraryPosition::Top,
                    },
                    controller_override: None,
                },
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
