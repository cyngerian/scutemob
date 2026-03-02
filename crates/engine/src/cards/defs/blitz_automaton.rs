// Blitz Automaton — {7} Artifact Creature — Construct 6/4 with Prototype {2}{R} — 3/2, Haste
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blitz-automaton"),
        name: "Blitz Automaton".to_string(),
        mana_cost: Some(ManaCost { generic: 7, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Construct"]),
        oracle_text:
            "Prototype {2}{R} — 3/2 (You may cast this spell with different mana cost, color, \
             and size. It keeps its abilities and types.)\n\
             Haste"
                .to_string(),
        abilities: vec![
            // CR 702.160: Keyword marker for quick presence-checking
            AbilityDefinition::Keyword(KeywordAbility::Prototype),
            // CR 702.160 / CR 718: Full Prototype data — prototype cost and P/T
            AbilityDefinition::Prototype {
                cost: ManaCost { generic: 2, red: 1, ..Default::default() },
                power: 3,
                toughness: 2,
            },
            AbilityDefinition::Keyword(KeywordAbility::Haste),
        ],
        power: Some(6),
        toughness: Some(4),
    }
}
