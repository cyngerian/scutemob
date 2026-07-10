// Yeva, Nature's Herald — {2}{G}{G}, Legendary Creature — Elf Shaman
// Flash
// You may cast green creature spells as though they had flash.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("yeva-natures-herald"),
        name: "Yeva, Nature's Herald".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Elf", "Shaman"]),
        oracle_text:
            "Flash\nYou may cast green creature spells as though they had flash.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // CR 601.3b: While Yeva is on the battlefield, controller may cast green
            // creature spells at instant speed. Registered as a WhileSourceOnBattlefield grant.
            AbilityDefinition::StaticFlashGrant { filter: FlashGrantFilter::GreenCreatures },
        ],
        ..Default::default()
    }
}
