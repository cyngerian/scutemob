// Ziatora, the Incinerator — {3}{B}{R}{G}, Legendary Creature — Demon Dragon 6/6
// Flying
// At the beginning of your end step, you may sacrifice another creature. When you do,
// Ziatora deals damage equal to that creature's power to any target and you create three
// Treasure tokens.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ziatora-the-incinerator"),
        name: "Ziatora, the Incinerator".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, red: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Demon", "Dragon"],
        ),
        oracle_text: "Flying\nAt the beginning of your end step, you may sacrifice another creature. When you do, Ziatora deals damage equal to that creature's power to any target and you create three Treasure tokens.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "At the beginning of your end step, you may sacrifice another creature.
            // When you do, deal damage equal to that creature's power to any target and
            // create three Treasure tokens."
            // DSL gaps:
            // 1. "You may sacrifice another creature" as optional cost in triggered ability
            //    not expressible (optional sacrifice as cost, not as effect).
            // 2. "When you do" — reflexive triggered ability off the sacrifice.
            // 3. Damage amount = sacrificed creature's power (dynamic LKI value) not
            //    expressible — no EffectAmount::SacrificedCreaturePower variant.
        ],
        ..Default::default()
    }
}
