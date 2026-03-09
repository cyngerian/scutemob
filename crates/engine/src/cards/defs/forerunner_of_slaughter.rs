// Forerunner of Slaughter — {B}{R}, Creature — Eldrazi Drone 3/2; Devoid.
// NOTE: "{1}: Target colorless creature gains haste until end of turn" is omitted
// because AbilityDefinition::Activated has no targets field (activated_ability_targets gap).
// TODO: Add the activated ability once Activated gains a TargetRequirement field.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forerunner-of-slaughter"),
        name: "Forerunner of Slaughter".to_string(),
        mana_cost: Some(ManaCost { black: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Eldrazi", "Drone"]),
        oracle_text: "Devoid (This card has no color.)\n{1}: Target colorless creature gains haste until end of turn.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Devoid),
        ],
        back_face: None,
    }
}
