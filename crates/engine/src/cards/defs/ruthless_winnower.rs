// Ruthless Winnower — {3}{B}{B}, Creature — Elf Rogue 4/4
// At the beginning of each player's upkeep, that player sacrifices a non-Elf
// creature of their choice.
//
// TODO: "non-Elf creature" filter — SacrificePermanents has no subtype exclusion.
// TODO: "each player's upkeep" — AtBeginningOfYourUpkeep only fires for controller.
// Approximated as controller's upkeep + unfiltered sacrifice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ruthless-winnower"),
        name: "Ruthless Winnower".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Rogue"]),
        oracle_text: "At the beginning of each player's upkeep, that player sacrifices a non-Elf creature of their choice.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // TODO: "each player's upkeep" needs AtBeginningOfEachPlayersUpkeep trigger.
            // Creature filter + non-Elf exclusion expressible but trigger gap remains;
            // keeping unfiltered until the trigger primitive is shipped.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::SacrificePermanents {
                    player: PlayerTarget::EachPlayer,
                    count: EffectAmount::Fixed(1),
                    filter: None,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
