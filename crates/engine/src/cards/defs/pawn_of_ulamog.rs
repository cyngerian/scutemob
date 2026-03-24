// Pawn of Ulamog — {1}{B}{B}, Creature — Vampire Shaman 2/2
// Whenever this creature or another nontoken creature you control dies, you may create
// a 0/1 colorless Eldrazi Spawn creature token. It has "Sacrifice this token: Add {C}."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pawn-of-ulamog"),
        name: "Pawn of Ulamog".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text: "Whenever Pawn of Ulamog or another nontoken creature you control dies, you may create a 0/1 colorless Eldrazi Spawn creature token. It has \"Sacrifice this creature: Add {C}.\"".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 603.10a: "Whenever Pawn of Ulamog or another nontoken creature you control dies."
            // PB-23: controller_you + nontoken_only filters via DeathTriggerFilter.
            // Note: "Pawn of Ulamog or another" = self included, so exclude_self: false.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: false, nontoken_only: true },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Eldrazi Spawn".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Eldrazi".to_string()), SubType("Spawn".to_string())].into_iter().collect(),
                        colors: im::OrdSet::new(),
                        power: 0,
                        toughness: 1,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: Some(ManaColor::Colorless),
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
