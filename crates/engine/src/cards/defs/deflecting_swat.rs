// Deflecting Swat — {2}{R}, Instant
// If you control a commander, you may cast this without paying its mana cost.
// You may choose new targets for target spell or ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("deflecting-swat"),
        name: "Deflecting Swat".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost.\nYou may choose new targets for target spell or ability.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Conditional free-cast + RetargetSpell not in DSL.
            effect: Effect::Nothing,
            targets: vec![TargetRequirement::TargetSpell],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
