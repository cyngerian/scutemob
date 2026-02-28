// 71. Boggart Ram-Gang — {R/G}{R/G}{R/G}, Creature — Goblin Warrior 3/3;
// Haste. Wither.
// Oracle cost is {R/G}{R/G}{R/G} (hybrid); simplified here to {R}{R}{R} because
// the ManaCost struct does not support hybrid mana symbols.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("boggart-ram-gang"),
        name: "Boggart Ram-Gang".to_string(),
        mana_cost: Some(ManaCost { red: 3, ..Default::default() }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Haste\nWither (This deals damage to creatures in the form of -1/-1 counters.)".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: Add KeywordAbility::Wither variant (CR 702.77a): damage dealt to
            // creatures is in the form of -1/-1 counters instead of marked damage.
        ],
    }
}
