// Goblin Wardriver — {R}{R}, Creature — Goblin Warrior 2/2
// Battle cry (Whenever this creature attacks, each other attacking creature gets
// +1/+0 until end of turn.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-wardriver"),
        name: "Goblin Wardriver".to_string(),
        mana_cost: Some(ManaCost {
            red: 2,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Battle cry (Whenever this creature attacks, each other attacking creature \
                      gets +1/+0 until end of turn.)"
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 702.92a: Battle cry — handled keyword (state::keyword_registry).
            AbilityDefinition::Keyword(KeywordAbility::BattleCry),
        ],
        ..Default::default()
    }
}
