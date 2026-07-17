// Brokers Ascendancy — {G}{W}{U}, Enchantment
// At the beginning of your end step, put a +1/+1 counter on each creature you control
// and a loyalty counter on each planeswalker you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brokers-ascendancy"),
        name: "Brokers Ascendancy".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            white: 1,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your end step, put a +1/+1 counter on each creature you \
                      control and a loyalty counter on each planeswalker you control."
            .to_string(),
        abilities: vec![
            // TODO: DSL gap — end step trigger (AtBeginningOfYourEndStep) with mass
            // counter placement on creatures + loyalty counters on planeswalkers.
            // TriggerCondition::AtBeginningOfYourEndStep not available for CardDef
            // triggered abilities.
        ],
        completeness: Completeness::partial(
            "Rewire only — no blocker. AtBeginningOfYourEndStep is live and used by 7 other defs \
             (atraxa_praetors_voice.rs et al.); mass +1/+1 counters follow \
             felidar_retreat.rs:67-71 (ForEach/EachCreatureYouControl -> AddCounter), and loyalty \
             counters follow ForEachTarget::EachPermanentMatching -> AddCounter{Loyalty}. Fully \
             authorable to Complete.",
        ),
        ..Default::default()
    }
}
