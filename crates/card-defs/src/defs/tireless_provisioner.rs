// Tireless Provisioner — {2}{G}, Creature — Elf Scout 3/2
// Landfall — Whenever a land you control enters, create a Food token or a Treasure token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tireless-provisioner"),
        name: "Tireless Provisioner".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Scout"]),
        oracle_text: "Landfall \u{2014} Whenever a land you control enters, create a Food token or a Treasure token.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // Landfall: create Treasure (Food or Treasure — using Treasure as default)
            // TODO: Player choice "Food or Treasure" not in DSL.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::CreateToken { spec: treasure_token_spec(1) },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong("'create a Food token or a Treasure token' is modeled as an unconditional Treasure. The player never gets the choice and never gets Food. Effect::Choose is non-interactive (effects/mod.rs:3190 executes choices.first()), so the modal choice has no correct expression today."),
        ..Default::default()
    }
}
