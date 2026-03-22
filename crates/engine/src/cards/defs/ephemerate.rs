// Ephemerate — {W}, Instant
// Exile target creature you control, then return it to the battlefield under its owner's control.
// Rebound (If you cast this spell from your hand, exile it as it resolves. At the beginning
// of your next upkeep, you may cast this card from exile without paying its mana cost.)
// TODO: Rebound keyword — not yet in the DSL KeywordAbility enum. Implementing the flicker
//   effect only; Rebound self-exile and upkeep re-cast are not modeled.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ephemerate"),
        name: "Ephemerate".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target creature you control, then return it to the battlefield under its owner's control.\nRebound (If you cast this spell from your hand, exile it as it resolves. At the beginning of your next upkeep, you may cast this card from exile without paying its mana cost.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Flicker {
                target: EffectTarget::DeclaredTarget { index: 0 },
                return_tapped: false,
            },
            targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                controller: TargetController::You,
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
