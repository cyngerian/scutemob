// Nocturnal Hunger — {2}{B}, Instant; Gift a Food; destroy target creature.
// If the gift wasn't promised, you lose 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nocturnal-hunger"),
        name: "Nocturnal Hunger".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Gift a Food (You may promise an opponent a gift as you cast this spell. If \
                      you do, they create a Food token before its other effects. It's an artifact \
                      with \"{2}, {T}, Sacrifice this token: You gain 3 life.\")\nDestroy target \
                      creature. If the gift wasn't promised, you lose 2 life."
            .to_string(),
        abilities: vec![
            // CR 702.174a: the Gift PRESENCE marker. Required, and its absence was a live
            // defect on this `Complete`, deck-legal def until PB-DX29 (`OOS-DX29-*`):
            // `casting.rs`'s gift block gates on
            // `chars.keywords.contains(&KeywordAbility::Gift)` BEFORE it looks the
            // `AbilityDefinition::Gift` below up, so an announced
            // `AdditionalCost::Gift { opponent }` was refused with "spell does not have
            // gift (CR 702.174a)" and the printed gift was unpayable by any client.
            // This is `ui2_additional_cost_roster::r3b`'s Squad defect one variant over,
            // inverted (cost present, marker absent) — see
            // `pb_dx29_additional_cost_roster::r2`, which now gates all five
            // marker/cost pairs rather than Squad alone.
            AbilityDefinition::Keyword(KeywordAbility::Gift),
            // CR 702.174a: Gift a Food — chosen opponent creates a Food token at resolution.
            AbilityDefinition::Gift {
                gift_type: GiftType::Food,
            },
            // Spell effect: destroy target creature; if gift was not promised, controller loses 2 life.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DestroyPermanent {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        cant_be_regenerated: false,
                    },
                    // CR 702.174b: "If the gift wasn't promised, you lose 2 life."
                    Effect::Conditional {
                        condition: Condition::GiftWasGiven,
                        if_true: Box::new(Effect::Sequence(vec![])),
                        if_false: Box::new(Effect::LoseLife {
                            player: PlayerTarget::Controller,
                            amount: EffectAmount::Fixed(2),
                        }),
                    },
                ]),
                targets: vec![TargetRequirement::TargetCreature],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
