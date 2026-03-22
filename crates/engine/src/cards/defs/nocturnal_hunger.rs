// Nocturnal Hunger — {2}{B}, Instant; Gift a Food; destroy target creature.
// If the gift wasn't promised, you lose 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nocturnal-hunger"),
        name: "Nocturnal Hunger".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Gift a Food (You may promise an opponent a gift as you cast this spell. \
If you do, they create a Food token before its other effects. It's an artifact with \
\"{2}, {T}, Sacrifice this token: You gain 3 life.\")\n\
Destroy target creature. If the gift wasn't promised, you lose 2 life."
            .to_string(),
        abilities: vec![
            // CR 702.174a: Gift a Food — chosen opponent creates a Food token at resolution.
            AbilityDefinition::Gift { gift_type: GiftType::Food },
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
