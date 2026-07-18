// Copperhorn Scout — {G}, Creature — Elf Scout 1/1
// Whenever this creature attacks, untap each other creature you control.
//
// PB-EF1: the "each OTHER creature you control" clause is `Effect::UntapAll` with
// `TargetFilter.exclude_self: true`. Before PB-EF1 the `UntapAll` executor ignored
// `exclude_self` (it matched a filter with no ObjectId in scope), so Copperhorn Scout
// would untap itself too. Now the executor honors it (effects/mod.rs, CR 109.1).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("copperhorn-scout"),
        name: "Copperhorn Scout".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Scout"]),
        oracle_text: "Whenever this creature attacks, untap each other creature you control."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            // CR 508.1: "Whenever this creature attacks" — the source is declared as an attacker.
            trigger_condition: TriggerCondition::WhenAttacks,
            effect: Effect::UntapAll {
                filter: TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    controller: TargetController::You,
                    // CR 109.1: "each OTHER creature you control" excludes Copperhorn Scout.
                    exclude_self: true,
                    ..Default::default()
                },
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
