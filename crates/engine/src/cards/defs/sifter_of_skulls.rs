// Sifter of Skulls — {3}{B}, Creature — Eldrazi 4/3
// Devoid. Whenever another nontoken creature you control dies, create a 1/1 colorless
// Eldrazi Scion creature token with "Sacrifice this token: Add {C}."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sifter-of-skulls"),
        name: "Sifter of Skulls".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Eldrazi"]),
        oracle_text: "Devoid (This card has no color.)\nWhenever another nontoken creature you control dies, create a 1/1 colorless Eldrazi Scion creature token. It has \"Sacrifice this token: Add {C}.\" ({C} represents colorless mana.)".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Devoid),
            // TODO: "Whenever another nontoken creature you control dies" —
            // WheneverCreatureDies trigger is overbroad (KI-5): it would trigger on
            // token deaths and on opponents' creatures. DSL lacks nontoken + controller-you
            // + self-exclusion filter on WhenDies triggers. Using TODO per W5 policy.
        ],
        ..Default::default()
    }
}
