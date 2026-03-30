// Bloodghast — {B}{B}, Creature — Vampire Spirit 2/1.
// Can't block; has haste if opponent at 10 or less life (conditional static);
// Landfall — whenever a land you control enters, may return from graveyard.
//
// CR 509.1b: "This creature can't block." — enforced via KeywordAbility::CantBlock.
//
// CR 604.2 / CR 613.1f (Layer 6): "This creature has haste as long as an opponent
// has 10 or less life." Implemented as a conditional static with Condition::OpponentLifeAtMost(10).
//
// CR 602.2 / PB-35 (TriggerZone::Graveyard): Landfall trigger from graveyard zone.
// When a land you control enters the battlefield while Bloodghast is in your graveyard,
// it returns to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodghast"),
        name: "Bloodghast".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Spirit"]),
        oracle_text: "This creature can't block.\nThis creature has haste as long as an opponent has 10 or less life.\nLandfall \u{2014} Whenever a land you control enters, you may return this card from your graveyard to the battlefield.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // CR 509.1b: "This creature can't block."
            AbilityDefinition::Keyword(KeywordAbility::CantBlock),
            // CR 604.2 / CR 613.1f (Layer 6): "This creature has haste as long as an opponent
            // has 10 or less life." Haste is granted conditionally when any living opponent
            // has a life total of 10 or below.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::OpponentLifeAtMost(10)),
                },
            },
            // PB-35 / CR 603.3 (TriggerZone::Graveyard): Landfall trigger from graveyard.
            // Whenever a land you control enters the battlefield while this is in your
            // graveyard, return it to the battlefield.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                // TODO: Oracle says "you may return" — currently non-optional (bot always returns).
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: Some(TriggerZone::Graveyard),
            },
        ],
        ..Default::default()
    }
}
