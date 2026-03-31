// Myriad Landscape — Land.
// "This land enters tapped."
// "{T}: Add {C}."
// "{2}, {T}, Sacrifice this land: Search your library for up to two basic land
// cards that share a land type, put them onto the battlefield tapped, then shuffle."
//
// TODO: The search ability requires a filter constraint "share a land type" —
// i.e., both found basic lands must have the same land subtype (e.g., both Forest,
// both Swamp, etc.). TargetFilter has no "must share subtype with another found card"
// field. Implementing without this constraint would allow any two basic lands
// regardless of type — wrong game state per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("myriad-landscape"),
        name: "Myriad Landscape".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {C}.\n{2}, {T}, Sacrifice this land: Search your library for up to two basic land cards that share a land type, put them onto the battlefield tapped, then shuffle.".to_string(),
        abilities: vec![
            // ETB tapped (always).
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield { filter: ObjectFilter::Any },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // TODO: {2}, {T}, Sacrifice this land: Search for up to two basic land cards
            // that share a land type, put them onto the battlefield tapped, then shuffle.
            // Needs a "paired same-subtype" filter that the current SearchLibrary DSL
            // does not support. Two sequential SearchLibrary calls with basic_land_filter()
            // would not enforce the shared-type constraint.
        ],
        ..Default::default()
    }
}
