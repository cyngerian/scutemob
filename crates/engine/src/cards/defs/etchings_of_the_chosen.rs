// Etchings of the Chosen — {1}{W}{B}, Enchantment
// As this enchantment enters, choose a creature type.
// Creatures you control of the chosen type get +1/+1.
// {1}, Sacrifice a creature of the chosen type: Target creature you control gains
// indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("etchings-of-the-chosen"),
        name: "Etchings of the Chosen".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "As this enchantment enters, choose a creature type.\nCreatures you control of the chosen type get +1/+1.\n{1}, Sacrifice a creature of the chosen type: Target creature you control gains indestructible until end of turn.".to_string(),
        abilities: vec![
            // "As this enters, choose a creature type"
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseCreatureType(SubType("Human".to_string())),
                is_self: true,
                unless_condition: None,
            },
            // TODO: Creatures you control of chosen type get +1/+1.
            // DSL gap: no dynamic subtype filter for continuous effects referencing
            // chosen_creature_type from source permanent. Would need EffectFilter::ChosenSubtype.
            // TODO: {1}, Sacrifice a creature of the chosen type: Target creature you control
            // gains indestructible until end of turn.
            // DSL gap: no Cost::SacrificeWithFilter(chosen subtype); no Effect::GainIndestructible.
        ],
        ..Default::default()
    }
}
