// Twilight Prophet — {2}{B}{B}, Creature — Vampire Cleric 2/4
// Flying, Ascend; upkeep trigger (with city's blessing): reveal top, draw it, opponents lose X, gain X
//
// TODO: Upkeep trigger conditioned on HasCitysBlessing requires:
//   1. TriggerCondition::AtBeginningOfYourUpkeep — EXISTS in DSL
//   2. intervening_if: Some(Condition::HasCitysBlessing) — EXISTS in DSL
//   3. Reveal top card and draw it — DrawCards EXISTS; reveal effect is cosmetic (no enforced reveal)
//   4. "Each opponent loses X life and you gain X life where X is that card's mana value" —
//      BLOCKED: no way to reference the mana value of "the card just drawn". EffectAmount::ManaValueOf
//      takes an EffectTarget, but there is no EffectTarget::LastDrawnCard or similar. The drawn
//      card's identity is not tracked in EffectContext. DSL gap.
//   Full implementation deferred until EffectAmount::ManaValueOfLastDrawnCard or equivalent is added.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("twilight-prophet"),
        name: "Twilight Prophet".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Cleric"]),
        oracle_text: "Flying\nAscend (If you control ten or more permanents, you get the city's blessing for the rest of the game.)\nAt the beginning of your upkeep, if you have the city's blessing, reveal the top card of your library and put it into your hand. Each opponent loses X life and you gain X life, where X is that card's mana value.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Ascend keyword — triggers city's blessing check
            AbilityDefinition::Keyword(KeywordAbility::Ascend),
            // TODO: Upkeep trigger conditioned on city's blessing, with drain-life based on
            // revealed card's mana value — requires EffectAmount::ManaValueOfRevealed or similar.
        ],
        ..Default::default()
    }
}
