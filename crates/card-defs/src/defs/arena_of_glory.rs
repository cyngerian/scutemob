// Arena of Glory — This land enters tapped unless you control a Mountain. {T}: Add {R}.
// {R}, {T}, Exert this land: Add {R}{R}. If that mana is spent on a creature spell, it
// gains haste until end of turn.
//
// PB-AC5: Cost::Exert (CR 701.43a/c activation-cost shape) is now implemented and
// validated via a dedicated unit test (see pb_ac5_alt_costs.rs), but this card remains
// BLOCKED: the "if that mana is spent on a creature spell, it gains haste" rider requires
// tracking WHICH permanent a specific unit of produced mana was later spent on, then
// applying a delayed effect to the resulting permanent. No such mana-spend-provenance ->
// delayed-effect primitive exists in the DSL (`ManaRestriction` only RESTRICTS what a
// mana unit can pay for -- it cannot trigger a follow-up effect on the paid-for object).
// This is a separate, still-open DSL gap; do not ship the third ability without it (would
// silently omit the haste rider -- wrong game state).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arena-of-glory"),
        name: "Arena of Glory".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped unless you control a Mountain.\n{T}: Add {R}.\n{R}, \
                      {T}, Exert this land: Add {R}{R}. If that mana is spent on a creature \
                      spell, it gains haste until end of turn. (An exerted permanent won't untap \
                      during your next untap step.)"
            .to_string(),
        abilities: vec![
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: Some(Condition::ControlLandWithSubtypes(vec![SubType(
                    "Mountain".to_string(),
                )])),
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 1, 0, 0),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
            // ENGINE-BLOCKED: {R}, {T}, Exert this land: Add {R}{R}. If that mana is
            // spent on a creature spell, it gains haste until end of turn. The Exert
            // activation cost itself (Cost::Exert) is implemented (PB-AC5), but the
            // mana-spend-conditional-haste rider has no DSL primitive (see file header).
        ],
        completeness: Completeness::partial(
            "{R}, {T}, Exert this land: Add {R}{R}. If that mana is spent on a creature spell, it \
             gains haste until end of turn. The...",
        ),
        ..Default::default()
    }
}
