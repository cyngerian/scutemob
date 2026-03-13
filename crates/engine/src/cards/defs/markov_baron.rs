// Markov Baron — {2}{B}, Creature — Vampire Noble 2/2
// Convoke, Lifelink, Madness {2}{B}
// TODO: DSL gap — static ability "Other Vampires you control get +1/+1."
//   (subtype-filtered continuous P/T bonus for "other" creatures not supported in card DSL)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("markov-baron"),
        name: "Markov Baron".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Noble"]),
        oracle_text: "Convoke (Your creatures can help cast this spell. Each creature you tap while casting this spell pays for {1} or one mana of that creature's color.)\nLifelink\nOther Vampires you control get +1/+1.\nMadness {2}{B} (If you discard this card, discard it into exile. When you do, cast it for its madness cost or put it into your graveyard.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
