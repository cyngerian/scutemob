// Urza's Incubator — {3}, Artifact
// As this artifact enters, choose a creature type.
// Creature spells of the chosen type cost {2} less to cast.
//
// TODO: SpellCostFilter::HasChosenSubtype — no variant exists to reference the chosen creature
//   type in a cost modifier. The ChooseCreatureType replacement works, but the cost reduction
//   cannot be wired to it. Note: affects ALL players (scope: AllPlayers), not just controller.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("urzas-incubator"),
        name: "Urza's Incubator".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "As this artifact enters, choose a creature type.\nCreature spells of the chosen type cost {2} less to cast.".to_string(),
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
