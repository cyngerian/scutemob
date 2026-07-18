// Ojutai, Soul of Winter — {5}{W}{U}, Legendary Creature — Dragon 5/6
// Flying, vigilance
// Whenever a Dragon you control attacks, tap target nonland permanent an opponent
//   controls. That permanent doesn't untap during its controller's next untap step.
//
// PB-EF3 (EF-W-MISS-10): the target was silently dropped before A1/A2 fixed
// enrich_spec_from_def to forward CardDef-declared targets into the runtime
// TriggeredAbilityDef and fixed the auto-target registry fallback to stop raw-indexing
// the wrong ability. Now Complete.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ojutai-soul-of-winter"),
        name: "Ojutai, Soul of Winter".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            white: 1,
            blue: 1,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Dragon"]),
        oracle_text: "Flying, vigilance\nWhenever a Dragon you control attacks, tap target \
                      nonland permanent an opponent controls. That permanent doesn't untap during \
                      its controller's next untap step."
            .to_string(),
        power: Some(5),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // CR 508.1m / CR 601.2c: "Whenever a Dragon you control attacks, tap target
            // nonland permanent an opponent controls. That permanent doesn't untap during
            // its controller's next untap step."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureYouControlAttacks {
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Dragon".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::Sequence(vec![
                    Effect::TapPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::PreventNextUntap {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    non_land: true,
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
