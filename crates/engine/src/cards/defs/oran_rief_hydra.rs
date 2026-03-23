// Oran-Rief Hydra — {4}{G}{G}, Creature — Hydra 5/5
// Trample
// Landfall — Whenever a land you control enters, put a +1/+1 counter on this creature.
// If that land is a Forest, put two +1/+1 counters on this creature instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oran-rief-hydra"),
        name: "Oran-Rief Hydra".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: creature_types(&["Hydra"]),
        oracle_text: "Trample\nLandfall — Whenever a land you control enters, put a +1/+1 counter on Oran-Rief Hydra. If that land is a Forest, put two +1/+1 counters on Oran-Rief Hydra instead.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // Landfall — put +1/+1 counter (or 2 if Forest).
            // Implementing the base case (1 counter) — Forest check is a DSL gap.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: DSL gap — "If that land is a Forest, put two counters instead."
            // Conditional based on entering permanent's subtype not in DSL.
        ],
        ..Default::default()
    }
}
