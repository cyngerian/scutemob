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
                // DSL GAP (PB-10 Finding 6): "You lose life equal to its mana value" requires
                // EffectAmount::ManaValueOfTarget (or similar) to express dynamic life loss based on
                // the reanimated card's converted mana cost. This variant does not exist yet.
                // The life loss is omitted — produces wrong game state (no life penalty on reanimate).
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    // "under your control" — CR 400.7 zone-change resets controller to owner,
                    // so we override here to give the caster control of the reanimated creature.
                    controller_override: Some(PlayerTarget::Controller),
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
