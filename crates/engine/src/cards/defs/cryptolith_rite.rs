// Cryptolith Rite — {1}{G} Enchantment
// Creatures you control have "{T}: Add one mana of any color."
//
// DSL gap: granting an activated ability to all creatures you control via a static effect
//   is not in DSL (LayerModification only supports GrantKeyword, not GrantActivatedAbility).
// W5 policy: cannot faithfully express this — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cryptolith-rite"),
        name: "Cryptolith Rite".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Creatures you control have \"{T}: Add one mana of any color.\"".to_string(),
        abilities: vec![
            // TODO: Creatures you control have "{T}: Add one mana of any color."
            //   (LayerModification lacks GrantActivatedAbility; only GrantKeyword exists)
        ],
        ..Default::default()
    }
}
