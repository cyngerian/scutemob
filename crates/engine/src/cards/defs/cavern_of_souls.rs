// Cavern of Souls — Land
// As this land enters, choose a creature type.
// {T}: Add {C}.
// {T}: Add one mana of any color. Spend this mana only to cast a creature spell
// of the chosen type, and that spell can't be countered.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cavern-of-souls"),
        name: "Cavern of Souls".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, choose a creature type.\n{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type, and that spell can't be countered.".to_string(),
        abilities: vec![
            // "As this enters, choose a creature type" — self-replacement effect (CR 614.1c)
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseCreatureType(SubType("Human".to_string())),
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // {T}: Add one mana of any color. Spend this mana only to cast a creature
            // spell of the chosen type.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColorRestricted {
                    player: PlayerTarget::Controller,
                    restriction: ManaRestriction::ChosenTypeCreaturesOnly,
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: "and that spell can't be countered" — uncounterability rider on the
            // mana restriction is not yet expressible. Requires linking mana source to
            // spell uncounterability at resolution time. Deferred to future primitive.
        ],
        ..Default::default()
    }
}
