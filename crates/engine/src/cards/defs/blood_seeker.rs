// Blood Seeker — {1}{B}, Creature — Vampire Shaman 1/1
// Whenever a creature an opponent controls enters, you may have that player lose 1 life.
//
// TODO: "you may have that player lose 1 life" — optional effect targeting the entering
//   creature's controller. Using mandatory drain from EachOpponent on any creature ETB
//   (slightly wrong — should be per-opponent, optional, triggered by their creatures only).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blood-seeker"),
        name: "Blood Seeker".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Whenever a creature an opponent controls enters, you may have that player lose 1 life.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    }),
                },
                effect: Effect::LoseLife {
                    player: PlayerTarget::EachOpponent,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
