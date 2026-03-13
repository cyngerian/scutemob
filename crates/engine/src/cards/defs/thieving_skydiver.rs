// Thieving Skydiver — {1}{U}, Creature — Merfolk Rogue 2/1
// Kicker {X} (X can't be 0); Flying
// ETB (if kicked): gain control of target artifact with mana value X or less; attach if Equipment
// TODO: Kicker X ETB conditional control-steal with mana-value filter not in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thieving-skydiver"),
        name: "Thieving Skydiver".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Merfolk", "Rogue"]),
        oracle_text: "Kicker {X}. X can't be 0. (You may pay an additional {X} as you cast this spell.)\nFlying\nWhen this creature enters, if it was kicked, gain control of target artifact with mana value X or less. If that artifact is an Equipment, attach it to this creature.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Kicker is handled at cast time by the engine's AltCostKind::Kicker support.
            // TODO: ETB trigger conditioned on "was kicked" that gains control of a target
            // artifact with mana value <= X (where X is kicker amount) and optionally
            // attaches it if Equipment — mana-value filter targeting and kicker-amount
            // variable not in DSL (targeted_trigger gap).
        ],
        ..Default::default()
    }
}
