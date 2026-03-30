// Crimestopper Sprite — {2}{U}, Creature — Faerie Detective 2/2
// Collect evidence 6 (optional additional cost); Flying; ETB tap target creature,
// and if evidence was collected, put a stun counter on it (CR 701.59).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crimestopper-sprite"),
        name: "Crimestopper Sprite".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Faerie".to_string()), SubType("Detective".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "As an additional cost to cast this spell, you may collect evidence 6. (Exile cards with total mana value 6 or greater from your graveyard.)\nFlying\nWhen this creature enters, tap target creature. If evidence was collected, put a stun counter on it. (If a permanent with a stun counter would become untapped, remove one from it instead.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 701.59a: Optional additional cost — exile cards from graveyard with total MV >= 6.
            AbilityDefinition::CollectEvidence { threshold: 6, mandatory: false },
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 701.59c: Linked ETB trigger — tap target creature; if evidence was collected,
            // put a stun counter on it. CounterType::Stun is not yet implemented in the engine;
            // TODO: add CounterType::Stun + stun-counter replacement effect and replace the
            // if_true branch below with:
            //   Effect::AddCounter { target: EffectTarget::DeclaredTarget { index: 0 },
            //                        counter: CounterType::Stun, count: 1 }
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::Conditional {
                        condition: Condition::EvidenceWasCollected,
                        // TODO: replace with AddCounter { counter: CounterType::Stun, count: 1 }
                        // once CounterType::Stun is added to the engine.
                        if_true: Box::new(Effect::Sequence(vec![])),
                        if_false: Box::new(Effect::Sequence(vec![])),
                    },
                ]),
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
    }
}
