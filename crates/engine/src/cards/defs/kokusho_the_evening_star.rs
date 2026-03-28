// Kokusho, the Evening Star — {4}{B}{B}, Legendary Creature — Dragon Spirit 5/5
// Flying
// When Kokusho dies, each opponent loses 5 life. You gain life equal to the life lost
// this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kokusho-the-evening-star"),
        name: "Kokusho, the Evening Star".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Spirit"],
        ),
        oracle_text: "Flying\nWhen Kokusho, the Evening Star dies, each opponent loses 5 life. You gain life equal to the life lost this way.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // When Kokusho dies, each opponent loses 5 life. "You gain life equal to
            // the life lost this way" — simplified to drain 5 from each opponent.
            // DrainLife handles the lose+gain pattern.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::DrainLife { amount: EffectAmount::Fixed(5) },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
