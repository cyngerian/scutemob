// Zurgo Bellstriker — {R}, Legendary Creature — Orc Warrior 2/2; Dash {1}{R};
// can't block creatures with power 2 or greater.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("zurgo-bellstriker"),
        name: "Zurgo Bellstriker".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Orc", "Warrior"]),
        oracle_text: "Dash {1}{R} (You may cast this spell for its dash cost. If you do, it \
                      gains haste, and it's returned from the battlefield to its owner's hand \
                      at the beginning of the next end step.)\n\
                      Zurgo Bellstriker can't block creatures with power 2 or greater."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Dash),
            AbilityDefinition::Dash {
                cost: ManaCost { generic: 1, red: 1, ..Default::default() },
            },
            // TODO: static ability "can't block creatures with power 2 or greater" —
            // no CantBlock { filter: PowerGTE(2) } variant exists yet in the DSL.
        ],
        ..Default::default()
    }
}
