// Strixhaven Stadium — {3}, Artifact
// {T}: Add {C}. Put a point counter on Strixhaven Stadium.
// Whenever a creature deals combat damage to you, remove a point counter.
// Whenever a creature you control deals combat damage to an opponent, put a point counter.
//   Then if 10+, remove all and that player loses the game.
//
// TODO: Complex triggered abilities — combat damage triggers with point counter tracking
//   and "loses the game" win condition. Implementing only the mana + counter ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("strixhaven-stadium"),
        name: "Strixhaven Stadium".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add {C}. Put a point counter on Strixhaven Stadium.\nWhenever a creature deals combat damage to you, remove a point counter from Strixhaven Stadium.\nWhenever a creature you control deals combat damage to an opponent, put a point counter on Strixhaven Stadium. Then if it has ten or more point counters on it, remove them all and that player loses the game.".to_string(),
        abilities: vec![
            // {T}: Add {C}. Put a point counter on this.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::Sequence(vec![
                    Effect::AddMana {
                        player: PlayerTarget::Controller,
                        mana: mana_pool(0, 0, 0, 0, 0, 1),
                    },
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::Custom("point".to_string()),
                        count: 1,
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // TODO: combat damage triggers (remove/add point counters, 10+ = lose game)
        ],
        ..Default::default()
    }
}
