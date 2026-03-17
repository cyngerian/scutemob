// Danitha Capashen, Paragon — {2}{W}, Legendary Creature — Human Knight 2/2
// First strike, vigilance, lifelink
// Aura and Equipment spells you cast cost {1} less to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("danitha-capashen-paragon"),
        name: "Danitha Capashen, Paragon".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Knight"]),
        oracle_text: "First strike, vigilance, lifelink\nAura and Equipment spells you cast cost {1} less to cast.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -1,
            filter: SpellCostFilter::AuraOrEquipment,
            scope: CostModifierScope::Controller,
            eminence: false,
            exclude_self: false,
        }],
        ..Default::default()
    }
}
