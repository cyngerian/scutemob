// Glare of Subdual — {2}{G}{W} Enchantment
// Tap an untapped creature you control: Tap target artifact or creature.
//
// DSL gap: "Tap an untapped creature you control" as activated ability cost requires
//   Cost::TapAnotherCreature (no such Cost variant; only Cost::Tap taps this permanent).
// W5 policy: cannot faithfully express this — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glare-of-subdual"),
        name: "Glare of Subdual".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Tap an untapped creature you control: Tap target artifact or creature.".to_string(),
        abilities: vec![
            // TODO: Tap an untapped creature you control: Tap target artifact or creature.
            //   (Cost enum lacks TapAnotherCreature variant)
        ],
        ..Default::default()
    }
}
