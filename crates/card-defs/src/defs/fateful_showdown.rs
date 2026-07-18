// Fateful Showdown — {2}{R}{R}, Instant
// Fateful Showdown deals damage to any target equal to the number of cards in your
// hand. Discard all the cards in your hand, then draw that many cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fateful-showdown"),
        name: "Fateful Showdown".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Fateful Showdown deals damage to any target equal to the number of cards in \
                      your hand. Discard all the cards in your hand, then draw that many cards."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 608.2h: the number of cards in hand is locked in as the spell resolves —
            // EffectAmount::HandSize is read at execution time, before WheelHand's own
            // (identical) hand-size snapshot for the discard/draw that follows.
            effect: Effect::Sequence(vec![
                Effect::DealDamage {
                    source: None,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::HandSize {
                        player: PlayerTarget::Controller,
                    },
                },
                // CR 701.9 / 121.1: discard the whole hand, then draw that many cards.
                Effect::WheelHand {
                    player: PlayerTarget::Controller,
                    disposal: WheelDisposal::Discard,
                    draw: WheelDraw::ThatMany,
                },
            ]),
            targets: vec![TargetRequirement::TargetAny],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
