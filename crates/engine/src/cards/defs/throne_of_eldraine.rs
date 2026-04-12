// Throne of Eldraine — {5}, Legendary Artifact
// As Throne of Eldraine enters, choose a color.
// {T}: Add four mana of the chosen color. Spend this mana only to cast monocolored
// spells of that color.
// {3}, {T}: Draw two cards. Spend only mana of the chosen color to activate this ability.
//
// TODO: "Choose a color" — ChosenColor designation not in DSL.
// TODO: Mana spending restriction "only monocolored spells of that color".
// TODO: Activation mana restriction "only mana of chosen color".
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("throne-of-eldraine"),
        name: "Throne of Eldraine".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: supertypes(&[SuperType::Legendary], &[CardType::Artifact]),
        oracle_text: "As Throne of Eldraine enters, choose a color.\n{T}: Add four mana of the chosen color. Spend this mana only to cast monocolored spells of that color.\n{3}, {T}: Draw two cards. Spend only mana of the chosen color to activate this ability.".to_string(),
        // TODO: ChosenColor designation not in DSL. No abilities expressible.
        abilities: vec![],
        ..Default::default()
    }
}
