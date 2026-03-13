// Etchings of the Chosen — {1}{W}{B}, Enchantment
// As this enchantment enters, choose a creature type.
// Creatures you control of the chosen type get +1/+1.
// {1}, Sacrifice a creature of the chosen type: Target creature you control gains indestructible until end of turn.
// TODO: DSL gap — "as this enters, choose a creature type" is an ETB choice effect that sets a
// remembered value used by two other abilities; no ChooseSubtype ETB effect or stored-choice
// continuous effect pattern exists in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("etchings-of-the-chosen"),
        name: "Etchings of the Chosen".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As this enchantment enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\n{1}, Sacrifice a creature of the chosen type: Target creature you control gains indestructible until end of turn.".to_string(),
        abilities: vec![
            // TODO: ETB replacement — choose a creature type as this enters.
            // DSL gap: no ChooseSubtype stored-choice effect.
            // TODO: static — creatures you control of chosen type get +1/+1.
            // DSL gap: no dynamic subtype filter continuous effect referencing stored choice.
            // TODO: activated — {1}, sacrifice a creature of chosen type: target creature gains indestructible until EOT.
            // DSL gap: no Cost::SacrificeWithFilter(chosen subtype); no Effect::GainIndestructible.
        ],
        ..Default::default()
    }
}
