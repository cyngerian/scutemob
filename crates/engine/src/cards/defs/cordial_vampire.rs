// Cordial Vampire — {B}{B}, Creature — Vampire 1/1
// Whenever this creature or another creature dies, put a +1/+1 counter on each Vampire
// you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cordial-vampire"),
        name: "Cordial Vampire".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever Cordial Vampire or another creature dies, put a +1/+1 counter on each Vampire you control.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // "Whenever this creature or another creature dies" = any creature dies.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: None, exclude_self: false, nontoken_only: false },
                effect: Effect::ForEach {
                    over: ForEachTarget::EachPermanentMatching(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        has_subtype: Some(SubType("Vampire".to_string())),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
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
