// Goblin Grappler — {R}, Creature — Goblin 1/1; Provoke
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-grappler"),
        name: "Goblin Grappler".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Provoke (Whenever this creature attacks, you may have target creature defending player controls untap and block it if able.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Provoke),
        ],
        back_face: None,
    }
}
