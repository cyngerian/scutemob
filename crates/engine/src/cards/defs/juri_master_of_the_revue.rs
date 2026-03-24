// Juri, Master of the Revue — {B}{R}, Legendary Creature — Human Shaman 1/1
// Whenever you sacrifice a permanent, put a +1/+1 counter on Juri.
// When Juri dies, it deals damage equal to its power to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("juri-master-of-the-revue"),
        name: "Juri, Master of the Revue".to_string(),
        mana_cost: Some(ManaCost { black: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Shaman"],
        ),
        oracle_text: "Whenever you sacrifice a permanent, put a +1/+1 counter on Juri.\nWhen Juri dies, it deals damage equal to its power to any target.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Whenever you sacrifice a permanent, put a +1/+1 counter on Juri.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouSacrifice {
                    filter: None,
                    player_filter: None,
                },
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: "When Juri dies, deals damage equal to its power to any target."
            // Needs EffectAmount::SourcePower.
        ],
        ..Default::default()
    }
}
