// Anafenza, Unyielding Lineage — {2}{W}, Legendary Creature — Spirit Soldier 2/2
// Flash
// First strike
// Whenever another nontoken creature you control dies, Anafenza endures 2.
// (Put two +1/+1 counters on it or create a 2/2 white Spirit creature token.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("anafenza-unyielding-lineage"),
        name: "Anafenza, Unyielding Lineage".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Spirit", "Soldier"]),
        oracle_text: "Flash\nFirst strike\nWhenever another nontoken creature you control dies, Anafenza endures 2. (Put two +1/+1 counters on it or create a 2/2 white Spirit creature token.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            // TODO: "Whenever another nontoken creature you control dies" — WhenDies with
            // nontoken + controller filter not in DSL. Endure keyword (put counters OR create
            // token) also not in DSL.
        ],
        ..Default::default()
    }
}
