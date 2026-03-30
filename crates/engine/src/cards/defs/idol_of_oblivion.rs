// Idol of Oblivion — {2}, Artifact
// {T}: Draw a card. Activate only if you created a token this turn.
// {8}, {T}, Sacrifice Idol of Oblivion: Create a 10/10 colorless Eldrazi creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("idol-of-oblivion"),
        name: "Idol of Oblivion".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Draw a card. Activate only if you created a token this turn.\n{8}, {T}, Sacrifice Idol of Oblivion: Create a 10/10 colorless Eldrazi creature token.".to_string(),
        abilities: vec![
            // {T}: Draw a card. Activate only if you created a token this turn.
            // TODO: Activation condition "created a token this turn" not in DSL.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None, // TODO: YouCreatedATokenThisTurn

                activation_zone: None,
            once_per_turn: false,
            },
            // {8}, {T}, Sacrifice: Create 10/10 Eldrazi token.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 8, ..Default::default() }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Eldrazi".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Eldrazi".to_string())].into_iter().collect(),
                        colors: im::OrdSet::new(),
                        power: 10,
                        toughness: 10,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
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
