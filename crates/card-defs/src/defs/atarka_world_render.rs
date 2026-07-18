// Atarka, World Render — {5}{R}{G}, Legendary Creature — Dragon 6/4
// Flying, trample
// Whenever a Dragon you control attacks, it gains double strike until end of turn.
//
// PB-EF4: the attack-trigger filter restricts to Dragons; the continuous double strike
// grant targets the attacking creature via EffectFilter::TriggeringCreature (CR 611.2a).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("atarka-world-render"),
        name: "Atarka, World Render".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying, trample\nWhenever a Dragon you control attacks, it gains double \
                      strike until end of turn."
            .to_string(),
        power: Some(6),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // CR 508.1m / CR 611.2a: "Whenever a Dragon you control attacks, it gains double
            // strike until end of turn." EffectFilter::TriggeringCreature aims the grant at
            // the attacking Dragon.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::DoubleStrike),
                        filter: EffectFilter::TriggeringCreature,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
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
