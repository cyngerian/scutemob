// Emeria, the Sky Ruin — Legendary Land
// This land enters tapped.
// At the beginning of your upkeep, if you control seven or more Plains, you may
// return target creature card from your graveyard to the battlefield.
// {T}: Add {W}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("emeria-the-sky-ruin"),
        name: "Emeria, the Sky Ruin".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "This land enters tapped.\nAt the beginning of your upkeep, if you control seven or more Plains, you may return target creature card from your graveyard to the battlefield.\n{T}: Add {W}.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // CR 603.4: Upkeep trigger — if you control 7+ Plains, return creature from GY to BF.
            // TODO: intervening_if needs Condition::YouControlNOrMorePermanentsWithSubtype { count: 7, subtype: Plains }
            // which doesn't exist yet. The graveyard targeting is implemented; the count threshold
            // condition is a separate DSL gap (deferred). Trigger fires unconditionally for now.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                },
                intervening_if: None, // TODO: Condition::YouControlNOrMorePermanentsWithSubtype
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
            },
            // {T}: Add {W}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(1, 0, 0, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
