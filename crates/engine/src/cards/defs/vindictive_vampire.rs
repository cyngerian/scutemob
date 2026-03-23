// Vindictive Vampire — {3}{B}, Creature — Vampire 2/3
// Whenever another creature you control dies, this creature deals 1 damage to each
// opponent and you gain 1 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vindictive-vampire"),
        name: "Vindictive Vampire".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever another creature you control dies, this creature deals 1 damage to each opponent and you gain 1 life.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            // WheneverCreatureDies is overbroad (fires on all creature deaths, not just
            // "another creature you control"). DSL gap — no controller/exclusion filter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You) },
                effect: Effect::Sequence(vec![
                    Effect::ForEach {
                        over: ForEachTarget::EachOpponent,
                        effect: Box::new(Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(1),
                        }),
                    },
                    Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
