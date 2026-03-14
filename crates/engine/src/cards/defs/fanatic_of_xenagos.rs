// Fanatic of Xenagos — {1}{R}{G}, Creature — Centaur Warrior 3/3; Trample; Tribute 1.
// CR 702.104: Tribute 1 — as this creature enters, an opponent may put 1 +1/+1 counter on it.
// If that opponent doesn't, it gets +1/+1 and gains haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fanatic-of-xenagos"),
        name: "Fanatic of Xenagos".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Centaur", "Warrior"]),
        oracle_text: "Trample\nTribute 1 (As this creature enters the battlefield, an opponent of your choice may put a +1/+1 counter on it. If that opponent doesn't, you get a benefit.)\nWhen Fanatic of Xenagos enters the battlefield, if tribute wasn't paid, it gets +1/+1 and gains haste until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            AbilityDefinition::Keyword(KeywordAbility::Tribute(1)),
            // CR 702.104b: fires inline at ETB only if tribute_was_paid == false.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::TributeNotPaid,
                effect: Effect::Sequence(vec![
                    // +1/+1 counter on self
                    Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    },
                    // Haste until end of turn (Layer 6)
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                            filter: EffectFilter::Source,
                            duration: EffectDuration::UntilEndOfTurn,
                        }),
                    },
                ]),
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
    }
}
