// Carrion Feeder — {B}, Creature — Zombie 1/1
// This creature can't block.
// Sacrifice a creature: Put a +1/+1 counter on this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("carrion-feeder"),
        name: "Carrion Feeder".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Zombie"]),
        oracle_text: "Carrion Feeder can't block.\nSacrifice a creature: Put a +1/+1 counter on Carrion Feeder.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 509.1b: "Carrion Feeder can't block."
            AbilityDefinition::Keyword(KeywordAbility::CantBlock),
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
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
            },
        ],
        ..Default::default()
    }
}
