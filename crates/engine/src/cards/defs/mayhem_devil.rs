// Mayhem Devil — {1}{B}{R} Creature — Devil 3/3
// Whenever a player sacrifices a permanent, this creature deals 1 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mayhem-devil"),
        name: "Mayhem Devil".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Devil"]),
        oracle_text: "Whenever a player sacrifices a permanent, Mayhem Devil deals 1 damage to any target.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // Whenever a player sacrifices a permanent, deal 1 damage to any target.
            // Using "any player" semantic via player_filter: Any.
            // TODO: "any target" — using each opponent as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouSacrifice {
                    filter: None,
                    player_filter: Some(TargetController::Any),
                },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::DealDamage {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
