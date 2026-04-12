// Poison-Tip Archer
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("poison-tip-archer"),
        name: "Poison-Tip Archer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Archer"]),
        oracle_text: "Reach
Deathtouch
Whenever another creature dies, each opponent loses 1 life.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // "Whenever another creature dies" = any creature except self.
            // WheneverCreatureDies triggers on ALL creatures (including your own + opponents').
            // This is correct for Poison-Tip Archer ("another creature" = any creature).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: None, exclude_self: true, nontoken_only: false, filter: None,
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
