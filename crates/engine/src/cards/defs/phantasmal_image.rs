// Phantasmal Image — {1}{U}, Creature — Illusion 0/0
// You may have this creature enter as a copy of any creature on the battlefield,
// except it's an Illusion in addition to its other types and it has "When this
// creature becomes the target of a spell or ability, sacrifice it."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("phantasmal-image"),
        name: "Phantasmal Image".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Illusion"]),
        oracle_text: "You may have this creature enter as a copy of any creature on the battlefield, except it's an Illusion in addition to its other types and it has \"When this creature becomes the target of a spell or ability, sacrifice it.\"".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            // TODO: "enter as a copy of any creature" — needs ETB replacement effect
            // with BecomeCopyOf + add Illusion subtype + add "sacrifice when targeted"
            // triggered ability. BecomeCopyOf infrastructure exists but ETB-replacement
            // clone choice is not expressible in the current DSL.
        ],
        ..Default::default()
    }
}
