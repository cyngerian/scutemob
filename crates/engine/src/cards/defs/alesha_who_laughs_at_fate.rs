// Alesha, Who Laughs at Fate — {1}{B}{R}, Legendary Creature — Human Warrior 2/2
// First strike
// Whenever Alesha attacks, put a +1/+1 counter on it.
// Raid — At the beginning of your end step, if you attacked this turn, return target
// creature card with mana value less than or equal to Alesha's power from your graveyard
// to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("alesha-who-laughs-at-fate"),
        name: "Alesha, Who Laughs at Fate".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Warrior"],
        ),
        oracle_text: "First strike\nWhenever Alesha attacks, put a +1/+1 counter on it.\nRaid — At the beginning of your end step, if you attacked this turn, return target creature card with mana value less than or equal to Alesha's power from your graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // Whenever Alesha attacks, put a +1/+1 counter on it.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: DSL gap — Raid: end step trigger with "if you attacked this turn"
            // intervening-if (Condition::YouAttackedThisTurn) + return creature from GY
            // with MV <= source power (TargetFilter with dynamic mana value comparison).
        ],
        ..Default::default()
    }
}
