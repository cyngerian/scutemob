// 71. Boggart Ram-Gang — {R/G}{R/G}{R/G}, Creature — Goblin Warrior 3/3;
// Haste. Wither.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boggart-ram-gang"),
        name: "Boggart Ram-Gang".to_string(),
        mana_cost: Some(ManaCost {
            hybrid: vec![
                HybridMana::ColorColor(ManaColor::Red, ManaColor::Green),
                HybridMana::ColorColor(ManaColor::Red, ManaColor::Green),
                HybridMana::ColorColor(ManaColor::Red, ManaColor::Green),
            ],
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Haste\nWither (This deals damage to creatures in the form of -1/-1 counters.)".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 702.80: Wither — damage dealt to creatures is in the form of -1/-1 counters.
            AbilityDefinition::Keyword(KeywordAbility::Wither),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
