// Lovestruck Beast // Heart's Desire — {2}{G} Creature — Beast Noble 5/5 + Adventure
//
// Main face: {2}{G} Beast Noble 5/5
// "Lovestruck Beast can't attack unless you control a 1/1 creature."
// Adventure face: "Heart's Desire" {G} Sorcery — Adventure
// "Create a 1/1 white Human creature token."
//
// TODO: Attack restriction "can't attack unless you control a 1/1 creature" — DSL gap:
// no ContinuousRestriction::CantAttackUnless with power/toughness filter on controlled creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("lovestruck-beast-hearts-desire"),
        name: "Lovestruck Beast // Heart's Desire".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Beast", "Noble"]),
        oracle_text: "Lovestruck Beast can't attack unless you control a 1/1 creature.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // TODO: "can't attack unless you control a 1/1 creature" —
            // DSL gap: no ContinuousRestriction::CantAttackUnlessYouControl1_1.
        ],
        // CR 715.2: Adventure face — Heart's Desire.
        adventure_face: Some(CardFace {
            name: "Heart's Desire".to_string(),
            mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
            types: TypeLine {
                card_types: [CardType::Sorcery].iter().copied().collect(),
                subtypes: [SubType("Adventure".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
                supertypes: Default::default(),
            },
            oracle_text: "Create a 1/1 white Human creature token.".to_string(),
            power: None,
            toughness: None,
            color_indicator: None,
            abilities: vec![AbilityDefinition::Spell {
                // CR 111.10: Create a 1/1 white Human creature token.
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Human".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::White].iter().copied().collect(),
                        card_types: [CardType::Creature].iter().copied().collect(),
                        subtypes: [SubType("Human".to_string())].iter().cloned().collect(),
                        count: 1,
                        tapped: false,
                        enters_attacking: false,
                        ..Default::default()
                    },
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            }],
        }),
        ..Default::default()
    }
}
