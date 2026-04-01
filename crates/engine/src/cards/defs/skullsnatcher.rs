// Skullsnatcher — {1}{B}, Creature — Rat Ninja 2/1
// Ninjutsu {B} ({B}, Return an unblocked attacker you control to hand: Put this card onto the
//   battlefield from your hand tapped and attacking.)
// Whenever this creature deals combat damage to a player, exile up to two target cards
//   from that player's graveyard.
//
// Note: "up to two" targeting requires two separate TargetCardInGraveyard entries or a
// "up to N" target variant, neither of which exists precisely. Two separate targets are used
// as an approximation (player may choose fewer targets). The "that player's graveyard"
// constraint uses TargetController::Opponent as an approximation — precise DamagedPlayer
// targeting is a known DSL gap.
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
            // that player's graveyard."
            // Approximation: two ExileObject effects on two targets from opponent's graveyard.
            // TODO: "that player's graveyard" → TargetController::DamagedPlayer not in DSL.
            // TODO: "up to two" → no UpToN target variant; using two required targets.
            AbilityDefinition::Triggered {
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
                targets: vec![
                    TargetRequirement::TargetCardInGraveyard(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    }),
                    TargetRequirement::TargetCardInGraveyard(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    }),
                ],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
