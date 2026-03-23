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
            // TODO: DSL gap — "Whenever you sacrifice a permanent" trigger condition
            // (WheneverYouSacrifice) does not exist.
            // TODO: DSL gap — "When Juri dies, deals damage equal to its power to any target."
            // Needs WhenThisDies trigger + EffectAmount::SourcePower + target any.
        ],
        ..Default::default()
    }
}
