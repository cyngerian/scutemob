// Juri, Master of the Revue — {B}{R}, Legendary Creature — Human Shaman 1/1
// Whenever you sacrifice a permanent, put a +1/+1 counter on Juri.
// When Juri dies, it deals damage equal to its power to any target.
//
// Death trigger uses `EffectAmount::SourcePowerAtLastKnownInformation` (PB-LKI-Power,
// CR 603.10a) to honor the 2020-11-10 ruling: use power as it last existed on the
// battlefield (boosted by +1/+1 counters before death). Damage of 0 or less resolves
// to 0 at the DealDamage boundary (CR 120.4). TargetAny allows p or creature targets.
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
                once_per_turn: false,
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

                modes: None,
                trigger_zone: None,
            },
            // When Juri dies, it deals damage equal to its power to any target.
            // CR 603.10a / Ruling 2020-11-10: power read from LKI snapshot
            // (boosted by accumulated +1/+1 counters before death).
            // CR 120.4: damage of 0 or less is reduced to 0 — Juri ruling explicitly notes this.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::SourcePowerAtLastKnownInformation,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
