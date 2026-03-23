// Betor, Kin to All — {2}{W}{B}{G}, Legendary Creature — Spirit Dragon 5/7
// Flying
// At the beginning of your end step, if creatures you control have total toughness
// 10 or greater, draw a card. Then if 20+, untap each creature. Then if 40+, each
// opponent loses half their life.
//
// TODO: Conditional toughness-threshold end step trigger too complex for DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("betor-kin-to-all"),
        name: "Betor, Kin to All".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Spirit", "Dragon"],
        ),
        oracle_text: "Flying\nAt the beginning of your end step, if creatures you control have total toughness 10 or greater, draw a card. Then if creatures you control have total toughness 20 or greater, untap each creature you control. Then if creatures you control have total toughness 40 or greater, each opponent loses half their life, rounded up.".to_string(),
        power: Some(5),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: Toughness-threshold tiered effects not expressible.
        ],
        ..Default::default()
    }
}
