// Shadow Alley Denizen — {B}, Creature — Vampire Rogue 1/1
// Whenever another black creature you control enters, target creature gains
// intimidate until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shadow-alley-denizen"),
        name: "Shadow Alley Denizen".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Vampire", "Rogue"]),
        oracle_text: "Whenever another black creature you control enters, target creature gains intimidate until end of turn. (It can't be blocked except by artifact creatures and/or creatures that share a color with it.)".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // TODO: "Whenever another black creature you control enters" — needs
            // ETB trigger with color filter (ETBTriggerFilter has creature_only,
            // controller_you, exclude_self but no color filter).
            // The effect (grant Intimidate until EOT) is expressible with
            // ApplyContinuousEffect + AddKeyword(Intimidate) + UntilEndOfTurn.
        ],
        ..Default::default()
    }
}
