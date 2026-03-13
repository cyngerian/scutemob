// Flawless Maneuver — {2}{W}, Instant; conditional free cast if commander controlled,
// creatures you control gain indestructible until end of turn.
// TODO: DSL gap — conditional free cast (if you control a commander, cast without paying
// mana cost) not expressible; mass keyword grant (AllCreatures indestructible until EOT)
// not expressible as a spell effect.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flawless-maneuver"),
        name: "Flawless Maneuver".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "If you control a commander, you may cast this spell without paying its mana cost.\nCreatures you control gain indestructible until end of turn.".to_string(),
        abilities: vec![],
        // TODO: conditional free cast based on commander presence; mass indestructible grant
        ..Default::default()
    }
}
