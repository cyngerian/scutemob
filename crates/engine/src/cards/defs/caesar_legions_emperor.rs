// Caesar, Legion's Emperor — {1}{R}{W}{B}, Legendary Creature — Human Soldier 4/4
// Whenever you attack, you may sacrifice another creature. When you do, choose two —
// * Create two 1/1 red and white Soldier creature tokens with haste that are tapped and attacking.
// * You draw a card and you lose 1 life.
// * Caesar deals damage equal to the number of creature tokens you control to target opponent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("caesar-legions-emperor"),
        name: "Caesar, Legion's Emperor".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, white: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Whenever you attack, you may sacrifice another creature. When you do, choose two \u{2014}\n\u{2022} Create two 1/1 red and white Soldier creature tokens with haste that are tapped and attacking.\n\u{2022} You draw a card and you lose 1 life.\n\u{2022} Caesar deals damage equal to the number of creature tokens you control to target opponent.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // Whenever you attack, create two 1/1 Soldier tokens tapped and attacking.
            // TODO: "may sacrifice + choose two modal" — complex reflexive trigger not expressible.
            // Partial: create Soldiers on attack as approximation of one modal option.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouAttack,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Soldier".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                        colors: [Color::Red, Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 2,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::Haste].into_iter().collect(),
                        tapped: true,
                        enters_attacking: true,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
