// Sheoldred, Whispering One — {5}{B}{B}, Legendary Creature — Phyrexian Praetor 6/6
// Swampwalk
// At the beginning of your upkeep, return target creature card from your graveyard to the battlefield.
// At the beginning of each opponent's upkeep, that player sacrifices a creature of their choice.
//
// TODO (DSL GAP): "At the beginning of each opponent's upkeep, that player sacrifices a creature"
// requires TriggerCondition::AtBeginningOfEachOpponentsUpkeep which does not exist.
// AtBeginningOfEachUpkeep exists but fires for all players and has no "that player" target.
// The opponent-sacrifice ability is omitted to avoid wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sheoldred-whispering-one"),
        name: "Sheoldred, Whispering One".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Phyrexian", "Praetor"],
        ),
        oracle_text: "Swampwalk (This creature can't be blocked as long as defending player controls a Swamp.)\nAt the beginning of your upkeep, return target creature card from your graveyard to the battlefield.\nAt the beginning of each opponent's upkeep, that player sacrifices a creature of their choice.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // CR 702.14a: Swampwalk — can't be blocked if defending player controls a Swamp.
            AbilityDefinition::Keyword(KeywordAbility::Landwalk(
                LandwalkType::BasicType(SubType("Swamp".to_string())),
            )),
            // CR 603.3: At the beginning of your upkeep, return target creature card from
            // your graveyard to the battlefield under your control.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: Some(PlayerTarget::Controller),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
                modes: None,
                trigger_zone: None,
            },
            // TODO (DSL GAP): "At the beginning of each opponent's upkeep, that player sacrifices
            // a creature of their choice." TriggerCondition::AtBeginningOfEachOpponentsUpkeep does
            // not exist. The ability is omitted rather than using AtBeginningOfEachUpkeep which
            // would incorrectly fire on your own upkeep too and lacks a "that player" effect target.
        ],
        ..Default::default()
    }
}
