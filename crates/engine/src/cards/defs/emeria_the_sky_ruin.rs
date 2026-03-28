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
            // DSL GAP (PB-10 Finding 4): "if you control seven or more Plains" is an intervening-if
            // (CR 603.4) that needs Condition::YouControlNOrMorePermanentsWithSubtype { count: 7,
            // subtype: "Plains" }, which does not exist yet. The graveyard targeting portion is
            // implemented. The condition is deferred — trigger fires unconditionally for now,
            // producing wrong game state (free reanimation every upkeep regardless of Plains count).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: None,
                },
                intervening_if: None, // TODO DSL gap: Condition::YouControlNOrMorePermanentsWithSubtype
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
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
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
