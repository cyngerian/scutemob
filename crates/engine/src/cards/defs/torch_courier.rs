// Torch Courier — {R}, Creature — Goblin 1/1
// Haste; Sacrifice: another target creature gains haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("torch-courier"),
        name: "Torch Courier".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Haste\nSacrifice this creature: Another target creature gains haste until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: Sacrifice: target creature gains haste until EOT — PB-5 (targeted activated)
            // Cost::SacrificeSelf available; blocked on targeted grant-keyword effect
        ],
        ..Default::default()
    }
}
