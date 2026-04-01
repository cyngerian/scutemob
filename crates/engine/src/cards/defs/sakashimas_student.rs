// Sakashima's Student — {2}{U}{U}, Creature — Human Ninja 0/0
// Ninjutsu {1}{U}
// You may have this creature enter as a copy of any creature on the battlefield,
// except it's a Ninja in addition to its other creature types.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sakashimas-student"),
        name: "Sakashima's Student".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 2, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {1}{U} ({1}{U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nYou may have this creature enter as a copy of any creature on the battlefield, except it's a Ninja in addition to its other creature types.".to_string(),
        power: Some(0),
        toughness: Some(0),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 1, blue: 1, ..Default::default() },
            },
            // TODO: "enter as a copy of any creature, except it's also a Ninja" — needs
            // ETB replacement effect with BecomeCopyOf + add-subtype. Using empty Triggered
            // for now; BecomeCopyOf infrastructure exists but ETB-replacement clone choice
            // is not expressible in the DSL.
        ],
        ..Default::default()
    }
}
