// Crossway Troublemakers — {5}{B}, Creature — Vampire 5/5
// Attacking Vampires you control have deathtouch and lifelink.
// Whenever a Vampire you control dies, you may pay 2 life. If you do, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crossway-troublemakers"),
        name: "Crossway Troublemakers".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Attacking Vampires you control have deathtouch and lifelink.\nWhenever a \
                      Vampire you control dies, you may pay 2 life. If you do, draw a card."
            .to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // CR 613.1f / CR 611.3a: "Attacking Vampires you control have deathtouch."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(SubType(
                        "Vampire".to_string(),
                    )),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.1f / CR 611.3a: "Attacking Vampires you control have lifelink."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Lifelink),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(SubType(
                        "Vampire".to_string(),
                    )),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // PB-AC2 (CR 118.12): "Whenever a Vampire you control dies, you may pay 2 life.
            // If you do, draw a card." — beneficial optional-pay wrapper.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Vampire".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::MayPayThenEffect {
                    cost: Cost::PayLife(2),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
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
