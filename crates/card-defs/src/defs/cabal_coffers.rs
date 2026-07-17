// Cabal Coffers — Land
// {2}, {T}: Add {B} for each Swamp you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cabal-coffers"),
        name: "Cabal Coffers".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{2}, {T}: Add {B} for each Swamp you control.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sequence(vec![
                Cost::Mana(ManaCost {
                    generic: 2,
                    ..Default::default()
                }),
                Cost::Tap,
            ]),
            effect: Effect::AddManaScaled {
                player: PlayerTarget::Controller,
                color: ManaColor::Black,
                count: EffectAmount::PermanentCount {
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        has_subtype: Some(SubType("Swamp".to_string())),
                        ..Default::default()
                    },
                    controller: PlayerTarget::Controller,
                },
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        completeness: Completeness::partial("CR 605.1a/605.3b: '{2},{T}: Add {B} for each Swamp' is a mana ability but is registered as a stack-using activated ability, so it cannot be activated via Command::TapForMana while paying for a spell and opponents get a priority window it should not grant. The AMOUNT is correct (probed: 4 Swamps -> +4 black). SR-34's mana_ability_lowering excludes Effect::AddManaScaled from the widened gate on purpose (Finding A): handle_tap_for_mana has no AddManaScaled branch and would read the registered `produces: {B:1}` marker literally, producing exactly one black. Un-blocked by SF-8 (memory/card-authoring/sr34-engine-findings-2026-07-17.md), after which the exclusion should be deleted."),
        ..Default::default()
    }
}
