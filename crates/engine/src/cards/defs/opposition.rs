// Opposition — {2}{U}{U} Enchantment
// Tap an untapped creature you control: Tap target artifact, creature, or land.
//
// DSL gap: "Tap an untapped creature you control" as activated ability cost requires
//   Cost::TapAnotherCreature (no such Cost variant; only Cost::Tap taps this permanent).
// W5 policy: cannot faithfully express tapping another creature as cost — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("opposition"),
        name: "Opposition".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Tap an untapped creature you control: Tap target artifact, creature, or land.".to_string(),
        abilities: vec![
            // TODO: Tap an untapped creature you control: Tap target artifact, creature, or land.
            //   (Cost enum lacks TapAnotherCreature variant)
        ],
        ..Default::default()
    }
}
