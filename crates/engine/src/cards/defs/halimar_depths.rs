// Halimar Depths
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("halimar-depths"),
        name: "Halimar Depths".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nWhen this land enters, look at the top three cards of your library, then put them back in any order.\n{T}: Add {U}.".to_string(),
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
            // When this land enters, look at the top three cards of your library, then put them back in any order.
            // TODO: DSL gap — no "rearrange top N" effect. Scry 3 is wrong (allows bottoming).
            // Need Effect::RearrangeTopN or similar. Using empty effect until DSL supports this.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![]),
                intervening_if: None,
                targets: vec![],
            },
            // {T}: Add {U}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 1, 0, 0, 0, 0) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
