// Maskwood Nexus — {4}, Artifact
// Creatures you control are every creature type. The same is true for creature
// spells you control and creature cards you own that aren't on the battlefield.
// {3}, {T}: Create a 2/2 blue Shapeshifter creature token with changeling.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("maskwood-nexus"),
        name: "Maskwood Nexus".to_string(),
        mana_cost: Some(ManaCost { generic: 4, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Creatures you control are every creature type. The same is true for creature spells you control and creature cards you own that aren't on the battlefield.\n{3}, {T}: Create a 2/2 blue Shapeshifter creature token with changeling. (It is every creature type.)".to_string(),
        abilities: vec![
            // TODO: Static — "Creatures you control are every creature type" — DSL lacks a
            //   "grant all subtypes / changeling" layer modification for permanents you control.
            // {3}, {T}: Create a 2/2 blue Shapeshifter token with changeling.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Shapeshifter".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Shapeshifter".to_string())].into_iter().collect(),
                        colors: [Color::Blue].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Changeling].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
