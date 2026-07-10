// Pact of the Serpent — {1}{B}{B}, Sorcery
// Choose a creature type. Target player draws X cards and loses X life, where X is
// the number of creatures they control of the chosen type.
//
// Per ruling (2021-02-05): "You choose the target player as you cast, but you don't
// choose the creature type until the spell resolves."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pact-of-the-serpent"),
        name: "Pact of the Serpent".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose a creature type. Target player draws X cards and loses X life, where X is the number of creatures they control of the chosen type.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // At resolution: choose creature type (sets ctx.chosen_creature_type).
                    Effect::ChooseCreatureType { default: SubType("Human".to_string()) },
                    // Draw X cards, where X = creatures of chosen type controlled by target player.
                    Effect::DrawCards {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::ChosenTypeCreatureCount {
                            controller: PlayerTarget::DeclaredTarget { index: 0 },
                        },
                    },
                    // Lose X life (same X).
                    Effect::LoseLife {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::ChosenTypeCreatureCount {
                            controller: PlayerTarget::DeclaredTarget { index: 0 },
                        },
                    },
                ]),
                targets: vec![TargetRequirement::TargetPlayer],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
