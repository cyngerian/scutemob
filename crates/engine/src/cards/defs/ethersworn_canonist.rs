// Ethersworn Canonist — {1}{W}, Artifact Creature — Human Cleric 2/2
// Each player who has cast a nonartifact spell this turn can't cast additional nonartifact spells.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ethersworn-canonist"),
        name: "Ethersworn Canonist".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: full_types(&[], &[CardType::Artifact, CardType::Creature], &["Human", "Cleric"]),
        oracle_text: "Each player who has cast a nonartifact spell this turn can't cast additional nonartifact spells.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::StaticRestriction {
                restriction: GameRestriction::MaxNonartifactSpellsPerTurn { max: 1 },
            },
        ],
        ..Default::default()
    }
}
