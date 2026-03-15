// Reanimate — {B} Sorcery
// Put target creature card from a graveyard onto the battlefield under your control.
// You lose life equal to that card's mana value.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reanimate"),
        name: "Reanimate".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Put target creature card from a graveyard onto the battlefield under your control. You lose life equal to its mana value.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // CR 115.1: Target creature card in any graveyard.
                // Moves target to battlefield under caster's control.
                // TODO: "You lose life equal to its mana value" — needs EffectAmount::ManaValueOfTarget
                // to express dynamic life loss. Defer to a future PB (dynamic target properties).
                // The graveyard targeting + return is the PB-10 primitive.
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                },
                targets: vec![TargetRequirement::TargetCardInGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
