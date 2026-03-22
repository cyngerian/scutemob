// Mortuary Mire
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mortuary-mire"),
        name: "Mortuary Mire".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nWhen this land enters, you may put target creature card from your graveyard on top of your library.\n{T}: Add {B}.".to_string(),
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
            // When this land enters, you may put target creature card from your graveyard on top of your library.
            // TODO: DSL gap — "you may" optional trigger not expressible. Currently mandatory if valid target exists.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Library { owner: PlayerTarget::Controller, position: LibraryPosition::Top },
                    controller_override: None,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                })],
            },
            // {T}: Add {B}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 1, 0, 0, 0) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
