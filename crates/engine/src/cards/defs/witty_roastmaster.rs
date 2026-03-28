// Witty Roastmaster — {2}{R}, Creature — Devil Citizen 3/2
// Alliance — Whenever another creature you control enters, this creature deals 1 damage to each opponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("witty-roastmaster"),
        name: "Witty Roastmaster".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: creature_types(&["Devil", "Citizen"]),
        oracle_text: "Alliance — Whenever another creature you control enters, this creature deals 1 damage to each opponent.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // Alliance — same pattern as Impact Tremors.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
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
