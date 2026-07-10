// Bartolomé del Presidio — {W}{B}, Legendary Creature — Vampire Knight 2/1
// Sacrifice another creature or artifact: Put a +1/+1 counter on Bartolomé del Presidio.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bartolome-del-presidio"),
        name: "Bartolomé del Presidio".to_string(),
        mana_cost: Some(ManaCost { white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Vampire", "Knight"],
        ),
        oracle_text: "Sacrifice another creature or artifact: Put a +1/+1 counter on Bartolomé del Presidio.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // CR 602.2: "Sacrifice another creature or artifact: Put a +1/+1 counter
            // on Bartolomé del Presidio."
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Artifact],
                    ..Default::default()
                }),
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
