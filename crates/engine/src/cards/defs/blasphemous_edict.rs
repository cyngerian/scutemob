// Blasphemous Edict — {B}{B}, Instant
// This spell costs {B}{B} less to cast if there are thirteen or more creatures
// on the battlefield.
// Each opponent sacrifices a creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blasphemous-edict"),
        name: "Blasphemous Edict".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "This spell costs {B}{B} less to cast if there are thirteen or more creatures on the battlefield.\nEach opponent sacrifices a creature.".to_string(),
        abilities: vec![
            // TODO: Conditional cost reduction (13+ creatures) not expressible.
            AbilityDefinition::Spell {
                // TODO: SacrificePermanents has no creature-only filter — opponent
                // sacrifices any permanent, not specifically a creature.
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachOpponent,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
