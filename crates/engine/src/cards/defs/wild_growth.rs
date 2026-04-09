// Wild Growth — {G}, Enchantment — Aura
// Enchant land
// Whenever enchanted land is tapped for mana, its controller adds an additional {G}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wild-growth"),
        name: "Wild Growth".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant land\nWhenever enchanted land is tapped for mana, its controller adds an additional {G}.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Land)),
            // CR 605.1b / CR 106.12a: "Whenever enchanted land is tapped for mana, add {G}."
            // EnchantedLand filter: fires when the land this Aura is attached to is tapped.
            // Triggered mana ability — resolves immediately (CR 605.4a).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenTappedForMana {
                    source_filter: ManaSourceFilter::EnchantedLand,
                },
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: ManaPool { green: 1, ..Default::default() },
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
