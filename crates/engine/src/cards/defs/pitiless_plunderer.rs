// Pitiless Plunderer — {3}{B}, Creature — Human Pirate 1/4.
// "Whenever another creature you control dies, create a Treasure token."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pitiless-plunderer"),
        name: "Pitiless Plunderer".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Pirate"]),
        oracle_text: "Whenever another creature you control dies, create a Treasure token.".to_string(),
        power: Some(1),
        toughness: Some(4),
        abilities: vec![
            // CR 603.10a: "Whenever another creature you control dies, create a Treasure token."
            // PB-23: controller_you + exclude_self filters via DeathTriggerFilter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: true,
                    nontoken_only: false,
                },
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
