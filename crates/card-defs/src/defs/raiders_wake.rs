// Raiders' Wake — {3}{B}, Enchantment
// Whenever an opponent discards a card, that player loses 2 life.
// Raid — At the beginning of your end step, if you attacked this turn, target opponent
// discards a card.
//
// The first ability IS authored below (TriggerCondition::WheneverOpponentDiscards and
// PlayerTarget::TriggeringPlayer both exist — the old marker claiming otherwise was stale).
//
// ENGINE-BLOCKED: the Raid half needs "target OPPONENT discards a card". PB-AC6 supplies the
// intervening-if (Condition::YouAttackedThisTurn), but TargetRequirement has TargetPlayer and
// no TargetOpponent, so authoring it would let the controller target themselves — an illegal
// target per the oracle, i.e. wrong game state. Omitted rather than approximated.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("raiders-wake"),
        name: "Raiders' Wake".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Whenever an opponent discards a card, that player loses 2 life.\nRaid — At \
                      the beginning of your end step, if you attacked this turn, target opponent \
                      discards a card."
            .to_string(),
        abilities: vec![
            // Whenever an opponent discards a card, that player loses 2 life.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverOpponentDiscards,
                effect: Effect::LoseLife {
                    player: PlayerTarget::TriggeringPlayer,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // ENGINE-BLOCKED: see file header — Raid end-step discard needs
            // TargetRequirement::TargetOpponent, which does not exist.
        ],
        completeness: Completeness::partial(
            "the Raid half needs 'target OPPONENT discards a card'. PB-AC6 supplies the \
             intervening-if...",
        ),
        ..Default::default()
    }
}
