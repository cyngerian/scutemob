// Blind Hunter — {2}{W}{B}, Creature — Bat 2/2; Flying, Haunt
// When Blind Hunter enters the battlefield or the haunted creature dies,
// each opponent loses 2 life and you gain 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blind-hunter"),
        name: "Blind Hunter".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Bat"]),
        oracle_text: "Flying, Haunt\nWhen Blind Hunter enters the battlefield or the haunted \
creature dies, each opponent loses 2 life and you gain 2 life."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 702.55a: Haunt keyword
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haunt),
            // ETB trigger: each opponent loses 2, controller gains 2
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DrainLife { amount: EffectAmount::Fixed(2) },
                intervening_if: None,
            },
            // CR 702.55c: haunted creature dies trigger (same effect)
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::HauntedCreatureDies,
                effect: Effect::DrainLife { amount: EffectAmount::Fixed(2) },
                intervening_if: None,
            },
        ],
        back_face: None,
    }
}
