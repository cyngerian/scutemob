// Umbral Collar Zealot — {1}{B}, Creature — Human Cleric 3/2
// Sacrifice another creature or artifact: Surveil 1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("umbral-collar-zealot"),
        name: "Umbral Collar Zealot".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "Sacrifice another creature or artifact: Surveil 1.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sacrifice(TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Artifact],
                    ..Default::default()
                }),
                effect: Effect::Surveil {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
