// Skullsnatcher — {1}{B}, Creature — Rat Ninja 2/1
// Ninjutsu {B} ({B}, Return an unblocked attacker you control to hand: Put this card onto the
//   battlefield from your hand tapped and attacking.)
// Whenever this creature deals combat damage to a player, exile up to two target cards
//   from that player's graveyard.
//
// PB-T `TargetRequirement::UpToN { count: 2, .. }` used for "up to two target cards" (was
// approximated as two mandatory targets — a wrong-game-state bug requiring exactly two
// legal graveyard cards to exist before the trigger could even resolve). The "that player's
// graveyard" constraint still uses TargetController::Opponent as an approximation — precise
// DamagedPlayer targeting on a TargetRequirement filter is a separate, unrelated DSL gap.
//
// ENGINE-BLOCKED (residual, documented — NOT fixed by this migration): this is a
// *triggered* ability. `abilities.rs`'s trigger auto-target selection routes any
// non-player-inner `UpToN` (including `TargetCardInGraveyard`) to `None` ("skip optional
// slots") rather than prompting the player. The trigger will fire correctly but always
// auto-select 0 targets (exiling nothing) until player-declared triggered-ability
// targeting is implemented (a broader feature, out of PB-AC4 scope — see pb-plan-AC4.md
// §A "Residual gap").
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skullsnatcher"),
        name: "Skullsnatcher".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Rat", "Ninja"]),
        oracle_text: "Ninjutsu {B} ({B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhenever this creature deals combat damage to a player, exile up to two target cards from that player's graveyard.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { black: 1, ..Default::default() },
            },
            // CR 510.3a: "Whenever this deals combat damage, exile up to two cards from
            // that player's graveyard." See file-level comment for the residual auto-target
            // gap on triggered abilities.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenDealsCombatDamageToPlayer,
                effect: Effect::Sequence(vec![
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    Effect::ExileObject {
                        target: EffectTarget::DeclaredTarget { index: 1 },
                    },
                ]),
                intervening_if: None,
                targets: vec![TargetRequirement::UpToN {
                    count: 2,
                    inner: Box::new(TargetRequirement::TargetCardInGraveyard(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    })),
                }],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
