// Hanweir Garrison — {2}{R} Creature — Human Soldier 2/3
// Whenever this creature attacks, create two 1/1 red Human creature tokens
// that are tapped and attacking.
// (Melds with Hanweir Battlements.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hanweir-garrison"),
        name: "Hanweir Garrison".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(
            &[],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Whenever Hanweir Garrison attacks, create two 1/1 red Human creature tokens that are tapped and attacking.\n(Melds with Hanweir Battlements.)".to_string(),
        abilities: vec![
            // CR 508.4: Attack trigger — create two 1/1 red Human tokens tapped and attacking.
            // Tokens inherit the attack target of the source creature (CR 508.4).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Human".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::Red].iter().copied().collect(),
                        card_types: [CardType::Creature].iter().copied().collect(),
                        subtypes: [SubType("Human".to_string())].iter().cloned().collect(),
                        count: 2,
                        tapped: true,
                        enters_attacking: true,
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        power: Some(2),
        toughness: Some(3),
        meld_pair: Some(MeldPair {
            pair_card_id: CardId("hanweir-battlements".to_string()),
            melded_card_id: CardId("hanweir-the-writhing-township".to_string()),
        }),
        ..Default::default()
    }
}
