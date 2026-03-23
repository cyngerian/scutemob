// Snap — {1}{U}, Instant
// Return target creature to its owner's hand. Untap up to two lands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("snap"),
        name: "Snap".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Return target creature to its owner's hand. Untap up to two lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::MoveZone {
                target: EffectTarget::DeclaredTarget { index: 0 },
                to: ZoneTarget::Hand {
                    owner: PlayerTarget::OwnerOf(Box::new(
                        EffectTarget::DeclaredTarget { index: 0 },
                    )),
                },
                controller_override: None,
            },
            // TODO: "Untap up to two lands" — requires untap-N-permanents effect with
            // land filter and "up to" choice.
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
