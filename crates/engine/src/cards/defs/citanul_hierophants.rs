// Citanul Hierophants — {3}{G} Creature — Human Druid 3/2
// Creatures you control have "{T}: Add {G}."
//
// DSL gap: granting an activated mana ability to all creatures you control via a static effect
//   is not in DSL (LayerModification only supports GrantKeyword, not GrantActivatedAbility).
// W5 policy: cannot faithfully express this — abilities: vec![].
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("citanul-hierophants"),
        name: "Citanul Hierophants".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: creature_types(&["Human", "Druid"]),
        oracle_text: "Creatures you control have \"{T}: Add {G}.\"".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            // TODO: Creatures you control have "{T}: Add {G}."
            //   (LayerModification lacks GrantActivatedAbility; only GrantKeyword exists)
        ],
        ..Default::default()
    }
}
