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
            // TODO: "Whenever you attack" trigger not in DSL.
            // TODO: Reflexive trigger + modal choice (choose two) not expressible.
        ],
        ..Default::default()
    }
}
