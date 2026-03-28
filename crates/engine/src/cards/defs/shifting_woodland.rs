// Shifting Woodland — Land
// This land enters tapped unless you control a Forest.
// {T}: Add {G}.
// Delirium — {2}{G}{G}: This land becomes a copy of target permanent card in your
// graveyard until end of turn. Activate only if there are four or more card types
// among cards in your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shifting-woodland"),
        name: "Shifting Woodland".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Forest.\n{T}: Add {G}.\nDelirium — {2}{G}{G}: This land becomes a copy of target permanent card in your graveyard until end of turn. Activate only if there are four or more card types among cards in your graveyard.".to_string(),
        abilities: vec![
            // ETB tapped unless you control a Forest.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType(
                    "Forest".to_string(),
                )])),
            },
            // {T}: Add {G}
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // Delirium — {2}{G}{G}: Become a copy of target permanent card in your graveyard
            // until end of turn. Activation condition: 4+ card types in graveyard.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 2,
                    green: 2,
                    ..Default::default()
                }),
                effect: Effect::BecomeCopyOf {
                    copier: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::UntilEndOfTurn,
                },
                timing_restriction: None,
                // TODO: TargetFilter lacks "permanent card" constraint. Currently allows
                // instants/sorceries as targets. Oracle says "target permanent card in
                // your graveyard." Needs TargetFilter.permanent_only or similar.
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(
                    TargetFilter::default(),
                )],
                activation_condition: Some(Condition::CardTypesInGraveyardAtLeast(4)),
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
