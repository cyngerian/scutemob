// Leyline of the Guildpact — {G/W}{G/U}{B/G}{R/G}, Enchantment
// If this card is in your opening hand, you may begin the game with it on the battlefield.
// Each nonland permanent you control is all colors.
// Lands you control are every basic land type in addition to their other types.
//
// TODO: "If this card is in your opening hand, begin the game with it on the battlefield" —
//   no Leyline/opening-hand-to-battlefield DSL primitive exists. DSL gap: need
//   ReplacementModification or special setup phase handling for leyline placement.
// TODO: "Each nonland permanent you control is all colors" — layer 5 color modification for
//   all nonland permanents you control. No ContinuousEffectDef modification for "all colors"
//   exists (LayerModification has no SetAllColors variant).
// TODO: "Lands you control are every basic land type" — layer 4 type addition for all lands.
//   No LayerModification::AddAllBasicLandTypes exists.
//   All three effects are DSL gaps. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("leyline-of-the-guildpact"),
        name: "Leyline of the Guildpact".to_string(),
        mana_cost: Some(ManaCost {
            hybrid: vec![
                HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
                HybridMana::ColorColor(ManaColor::Green, ManaColor::Blue),
                HybridMana::ColorColor(ManaColor::Black, ManaColor::Green),
                HybridMana::ColorColor(ManaColor::Red, ManaColor::Green),
            ],
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If this card is in your opening hand, you may begin the game with it on the battlefield.\nEach nonland permanent you control is all colors.\nLands you control are every basic land type in addition to their other types.".to_string(),
        abilities: vec![
            // TODO: Opening-hand leyline placement — DSL gap (no opening-hand-to-battlefield primitive).
            // TODO: "Each nonland permanent you control is all colors" — layer 5 static.
            //   DSL gap: no LayerModification::SetAllColors variant.
            // TODO: "Lands you control are every basic land type" — layer 4 static.
            //   DSL gap: no LayerModification::AddAllBasicLandTypes variant.
        ],
        ..Default::default()
    }
}
