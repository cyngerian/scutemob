// Thalia, Guardian of Thraben — {1}{W}, Legendary Creature — Human Soldier 2/1
// First strike
// Noncreature spells cost {1} more to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thalia-guardian-of-thraben"),
        name: "Thalia, Guardian of Thraben".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "First strike\nNoncreature spells cost {1} more to cast.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
        ],
        spell_cost_modifiers: vec![SpellCostModifier {
            change: 1,
            filter: SpellCostFilter::NonCreature,
            scope: CostModifierScope::AllPlayers,
            eminence: false,
            exclude_self: false,
            colored_mana_reduction: None,
        }],
        ..Default::default()
    }
}
