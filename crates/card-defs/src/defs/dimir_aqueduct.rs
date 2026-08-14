// Dimir Aqueduct
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dimir-aqueduct"),
        name: "Dimir Aqueduct".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nWhen this land enters, return a land you control \
                      to its owner's hand.\n{T}: Add {U}{B}."
            .to_string(),
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
            // CR 603.1 / CR 115.10 (PB-DX28): When this land enters, return a land you
            // control to its owner's hand. Printed with no "target" — a resolution-time
            // UNTARGETED choice (CR 115.10), not a declared target: unaffected by
            // hexproof/shroud/protection, and re-chosen at resolution if the original
            // candidate leaves in response (no CR 608.2b fizzle window).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::ChosenObject {
                        zone: ChoiceZone::Battlefield,
                        filter: Box::new(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            controller: TargetController::You,
                            ..Default::default()
                        }),
                        count: 1,
                        up_to: false,
                    },
                    to: ZoneTarget::Hand {
                        owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::ChosenObject {
                            zone: ChoiceZone::Battlefield,
                            filter: Box::new(TargetFilter {
                                has_card_type: Some(CardType::Land),
                                controller: TargetController::You,
                                ..Default::default()
                            }),
                            count: 1,
                            up_to: false,
                        })),
                    },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 1, 1, 0, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
