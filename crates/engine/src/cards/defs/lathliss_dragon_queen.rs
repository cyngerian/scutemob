// Lathliss, Dragon Queen — {4}{R}{R}, Legendary Creature — Dragon 6/6
// Flying
// Whenever another nontoken Dragon you control enters, create a 5/5 red Dragon creature
// token with flying.
// {1}{R}: Dragons you control get +1/+0 until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lathliss-dragon-queen"),
        name: "Lathliss, Dragon Queen".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying\nWhenever another nontoken Dragon you control enters, create a 5/5 red Dragon creature token with flying.\n{1}{R}: Dragons you control get +1/+0 until end of turn.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "Whenever another nontoken Dragon enters" — subtype-filtered ETB trigger not in DSL
            // TODO: "{1}{R}: Dragons you control get +1/+0 until EOT" — filtered pump not in DSL
        ],
        ..Default::default()
    }
}
