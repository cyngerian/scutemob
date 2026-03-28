// Grim Hireling — {3}{B}, Creature — Tiefling Rogue 3/2
// Whenever one or more creatures you control deal combat damage to a player, create two
// Treasure tokens.
// {B}, Sacrifice X Treasures: Target creature gets -X/-X until end of turn. Activate only
// as a sorcery.
//
// TODO: "{B}, Sacrifice X Treasures: Target creature gets -X/-X" — X-cost activated
//   ability with variable sacrifice count is not expressible in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grim-hireling"),
        name: "Grim Hireling".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Tiefling", "Rogue"]),
        oracle_text: "Whenever one or more creatures you control deal combat damage to a player, create two Treasure tokens.\n{B}, Sacrifice X Treasures: Target creature gets -X/-X until end of turn. Activate only as a sorcery.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // CR 510.3a / CR 603.2c: "Whenever one or more creatures you control deal combat
            // damage to a player, create two Treasure tokens." — batch trigger.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenOneOrMoreCreaturesYouControlDealCombatDamageToPlayer { filter: None },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "{B}, Sacrifice X Treasures: Target creature gets -X/-X" — X-cost
            //   activated ability with variable sacrifice count not expressible in DSL.
        ],
        ..Default::default()
    }
}
