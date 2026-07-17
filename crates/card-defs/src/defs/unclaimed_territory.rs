// Unclaimed Territory — Land
// As this land enters, choose a creature type.
// {T}: Add {C}.
// {T}: Add one mana of any color. Spend this mana only to cast a creature spell
// of the chosen type.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("unclaimed-territory"),
        name: "Unclaimed Territory".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "As this land enters, choose a creature type.\n{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type.".to_string(),
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
            // {T}: Add {C}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {T}: Add one mana of any color (restricted to chosen creature type spells)
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColorRestricted {
                    player: PlayerTarget::Controller,
                    restriction: ManaRestriction::ChosenTypeCreaturesOnly,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        completeness: Completeness::known_wrong("CR 106.1b: '{T}: Add one mana of any color. Spend this mana only to cast a creature spell of the chosen type' adds one COLORLESS mana. The RESTRICTION is honoured (probed: pool.restricted = [Colorless x1 (CreatureSpellsOnly)]) but colorless is not a color. Also CR 605.1a/605.3b: Effect::AddManaAnyColorRestricted has no try_as_tap_mana_ability arm, so despite a bare Cost::Tap this is a stack-using activated ability. ChooseCreatureType is NOT hardcoded (replacement.rs picks the most common creature subtype on board). The '{T}: Add {C}' ability is correct."),
        ..Default::default()
    }
}
