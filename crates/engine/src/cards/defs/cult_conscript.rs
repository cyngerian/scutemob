// Cult Conscript
// {1}{B}: Return this card from your graveyard to the battlefield.
// Activate only if a non-Skeleton creature died under your control this turn.
//
// CR 602.2 / PB-35: Graveyard-activated ability. The "activate only if a non-Skeleton
// creature died under your control this turn" condition is a DSL gap (no Condition variant
// for creature-died-this-turn checks). Implementing without the activation condition as
// a known approximation. The core graveyard activation works correctly.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cult-conscript"),
        name: "Cult Conscript".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Skeleton", "Warrior"]),
        oracle_text: "This creature enters tapped.\n{1}{B}: Return this card from your graveyard to the battlefield. Activate only if a non-Skeleton creature died under your control this turn.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this creature enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // CR 602.2 / PB-35: "{1}{B}: Return this card from your graveyard to the
            // battlefield." — activated from the graveyard zone.
            // TODO: "Activate only if a non-Skeleton creature died under your control
            // this turn" condition requires a Condition::NonSkeletonCreatureDiedThisTurn
            // variant not yet in the DSL. The ability fires unconditionally as an
            // approximation. This is a known DSL gap deferred to PB-37.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, black: 1, ..Default::default() }),
                effect: Effect::MoveZone {
                    target: EffectTarget::Source,
                    to: ZoneTarget::Battlefield { tapped: false },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: Some(ActivationZone::Graveyard),
            once_per_turn: false,
            },
        ],
        power: Some(2),
        toughness: Some(1),
        ..Default::default()
    }
}
