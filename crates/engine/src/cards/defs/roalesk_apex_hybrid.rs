// Roalesk, Apex Hybrid — {2}{G}{G}{U}, Legendary Creature — Human Mutant 4/5
// Flying, trample
// When Roalesk enters, put two +1/+1 counters on another target creature you control.
// When Roalesk dies, proliferate, then proliferate again.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("roalesk-apex-hybrid"),
        name: "Roalesk, Apex Hybrid".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Mutant"],
        ),
        oracle_text: "Flying, trample\nWhen Roalesk, Apex Hybrid enters, put two +1/+1 counters on another target creature you control.\nWhen Roalesk, Apex Hybrid dies, proliferate, then proliferate again.".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // ETB: put two +1/+1 counters on another target creature you control.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddCounter {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    counter: CounterType::PlusOnePlusOne,
                    count: 2,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
            },
            // TODO: DSL gap — "When Roalesk dies, proliferate, then proliferate again."
            // WhenThisDies trigger + Effect::Proliferate (twice).
        ],
        ..Default::default()
    }
}
