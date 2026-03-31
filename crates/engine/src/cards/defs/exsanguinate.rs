// Exsanguinate — {X}{B}{B}, Sorcery
// Each opponent loses X life. You gain life equal to the life lost this way.
//
// {X} spells use EffectAmount::XValue. DrainLife captures the lose+gain pattern.
// TODO: {X} mana cost — EffectAmount::XValue not wired for DrainLife; using Fixed(0) would
// be wrong (W5). Abilities left empty until XValue is expressible in DrainLife amount.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exsanguinate"),
        name: "Exsanguinate".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Each opponent loses X life. You gain life equal to the life lost this way.".to_string(),
        abilities: vec![
            // TODO: X spells — "each opponent loses X life, you gain that much."
            // DrainLife exists but EffectAmount::XValue is not yet wired into DrainLife's
            // amount resolver. Once XValue is supported there, implement as:
            //   Effect::DrainLife { amount: EffectAmount::XValue }
            // W5: a Fixed(0) placeholder would produce wrong game state — omitted.
        ],
        ..Default::default()
    }
}
