// Thalia, Guardian of Thraben — {1}{W}, Legendary Creature — Human Soldier 2/1
// First strike
// Noncreature spells cost {1} more to cast.
// TODO: DSL gap — "noncreature spells cost {1} more" is a static cost-increase continuous effect
// applied to opponents' (and controller's) spells; no SpellCostIncrease continuous effect exists.
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
            // TODO: static — noncreature spells cost {1} more to cast.
            // DSL gap: no SpellCostIncrease continuous effect with card type filter.
        ],
        ..Default::default()
    }
}
