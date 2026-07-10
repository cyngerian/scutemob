// Kindred Dominance — {5}{B}{B} Sorcery
// Choose a creature type. Destroy all creatures that aren't of the chosen type.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kindred-dominance"),
        name: "Kindred Dominance".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose a creature type. Destroy all creatures that aren't of the chosen type.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    // First: choose a creature type (sets ctx.chosen_creature_type at resolution).
                    Effect::ChooseCreatureType { default: SubType("Human".to_string()) },
                    // Then: destroy all creatures that aren't of the chosen type.
                    // exclude_chosen_subtype: true means "skip creatures WITH the chosen type".
                    Effect::DestroyAll {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            exclude_chosen_subtype: true,
                            ..Default::default()
                        },
                        cant_be_regenerated: false,
                    },
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
