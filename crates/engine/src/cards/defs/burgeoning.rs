// Burgeoning — {G}, Enchantment
// Whenever an opponent plays a land, you may put a land card from your hand onto the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("burgeoning"),
        name: "Burgeoning".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent plays a land, you may put a land card from your hand onto the battlefield.".to_string(),
        abilities: vec![
            // CR 305.1: Whenever an opponent plays a land (special action, not via effect).
            // CR 305.4: Effect puts land onto battlefield without counting as a land play.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverOpponentPlaysLand,
                effect: Effect::PutLandFromHandOntoBattlefield { tapped: false },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
