// Valakut, the Molten Pinnacle
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("valakut-the-molten-pinnacle"),
        name: "Valakut, the Molten Pinnacle".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\nWhenever a Mountain you control enters, if you \
                      control at least five other Mountains, you may have this land deal 3 damage \
                      to any target.\n{T}: Add {R}."
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
            // {T}: Add {R}.
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
            // TODO: Triggered — Whenever a Mountain you control enters, if you control at
            // least five other Mountains, deal 3 damage to any target.
            // DSL gap: WheneverPermanentEntersBattlefield with subtype filter + intervening-if
            // count condition + "any target" targeting.
        ],
        completeness: Completeness::partial(
            "the trigger's shape is now expressible (WheneverPermanentEntersBattlefield{filter: \
             has_subtype Mountain, controller: You} + intervening_if: Option<Condition>), but two \
             gaps remain: (1) 'you MAY have this land deal 3 damage' — Effect::Choose always \
             executes choices[0] (effects/mod.rs:3190), so the damage would be mandatory; (2) \
             Condition::ControlAtLeastNOtherLandsWithSubtype excludes ctx.source (Valakut), not \
             the ENTERING Mountain, so it counts the just-entered Mountain toward 'five other \
             Mountains' — off by one. ETB-tapped and the mana ability are complete.",
        ),
        ..Default::default()
    }
}
