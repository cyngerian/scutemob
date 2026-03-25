// Urza's Incubator — {3}, Artifact
// As this artifact enters, choose a creature type.
// Creature spells of the chosen type cost {2} less to cast.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("urzas-incubator"),
        name: "Urza's Incubator".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As this artifact enters, choose a creature type.\nCreature spells of the chosen type cost {2} less to cast.".to_string(),
        // CR 601.2f: Creature spells of the chosen type cost {2} less (all players).
        // Uses HasChosenCreatureSubtype — reads chosen_creature_type from source object at cast time.
        // Note: "Creature spells of the chosen type cost {2} less" has no "you cast" qualifier,
        // so scope is AllPlayers per oracle text.
        spell_cost_modifiers: vec![SpellCostModifier {
            change: -2,
            filter: SpellCostFilter::HasChosenCreatureSubtype,
            scope: CostModifierScope::AllPlayers,
            eminence: false,
            exclude_self: false,
        }],
        abilities: vec![
            // "As this enters, choose a creature type" — self-replacement (CR 614.1c)
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::ChooseCreatureType(SubType("Human".to_string())),
                is_self: true,
                unless_condition: None,
            },
        ],
        ..Default::default()
    }
}
