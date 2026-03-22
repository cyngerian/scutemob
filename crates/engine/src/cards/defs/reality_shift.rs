// Reality Shift — {1}{U}, Instant
// Exile target creature. Its controller manifests the top card of their library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reality-shift"),
        name: "Reality Shift".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target creature. Its controller manifests the top card of their library. (That player puts the top card of their library onto the battlefield face down as a 2/2 creature. If it's a creature card, it can be turned face up any time for its mana cost.)".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 701.5: Exile target creature.
                Effect::ExileObject {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                // CR 701.40: Its controller (not the spell's controller) manifests the top card.
                Effect::Manifest {
                    player: PlayerTarget::ControllerOf(Box::new(
                        EffectTarget::DeclaredTarget { index: 0 },
                    )),
                },
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
