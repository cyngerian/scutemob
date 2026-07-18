// Spiteful Banditry — {X}{R}{R}, Enchantment
// When this enchantment enters, it deals X damage to each creature.
// Whenever one or more creatures your opponents control die, you create a Treasure
// token. This ability triggers only once each turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("spiteful-banditry"),
        name: "Spiteful Banditry".to_string(),
        mana_cost: Some(ManaCost {
            red: 2,
            x_count: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When this enchantment enters, it deals X damage to each creature.\nWhenever \
                      one or more creatures your opponents control die, you create a Treasure \
                      token. This ability triggers only once each turn."
            .to_string(),
        abilities: vec![
            // CR 107.3m: "When this enchantment enters, it deals X damage to each creature."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCreature,
                    effect: Box::new(Effect::DealDamage {
                        source: None,
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        amount: EffectAmount::XValue,
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // "Whenever one or more creatures your opponents control die, you create a
            // Treasure token. This ability triggers only once each turn." (CR 603.2h / PB-AC1
            // once_per_turn throttle — mirrors morbid_opportunist.rs.) Mandatory: the oracle
            // text carries no "may".
            AbilityDefinition::Triggered {
                once_per_turn: true,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::Opponent),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
