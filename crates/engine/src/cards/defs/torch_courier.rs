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
            // TODO: DSL gap — activated ability "Sacrifice this creature: another target creature
            // gains haste until end of turn" requires targeting + ApplyContinuousEffect granting
            // Haste until EOT; targeted activated abilities are not expressible in the DSL.
        ],
        ..Default::default()
    }
}
