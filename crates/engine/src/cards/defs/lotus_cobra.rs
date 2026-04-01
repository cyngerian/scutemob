// Lotus Cobra — {1}{G}, Creature — Snake 2/1
// Landfall — Whenever a land you control enters, add one mana of any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lotus-cobra"),
        name: "Lotus Cobra".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Snake"]),
        oracle_text: "Landfall — Whenever a land you control enters, add one mana of any color.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // Landfall — Whenever a land you control enters, add one mana of any color.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
