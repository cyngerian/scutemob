// Vraska, Betrayal's Sting — {4}{B}{B/P} Legendary Planeswalker — Vraska
// Compleated ({B/P} can be paid with {B} or 2 life. If life was paid, enters
// with two fewer loyalty counters.)
// 0: You draw a card and lose 1 life. Proliferate.
// -2: Target creature becomes a Treasure artifact with "{T}, Sacrifice this
//     artifact: Add one mana of any color" and loses all other card types and abilities.
// -9: If target player has fewer than nine poison counters, they get a number of
//     poison counters equal to the difference.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vraska-betrayals-sting"),
        name: "Vraska, Betrayal's Sting".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            black: 1,
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Black)],
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Vraska"],
        ),
        oracle_text: "Compleated ({B/P} can be paid with {B} or 2 life. If life was paid, this planeswalker enters with two fewer loyalty counters.)\n0: You draw a card and lose 1 life. Proliferate.\n\u{2212}2: Target creature becomes a Treasure artifact with \"{T}, Sacrifice this artifact: Add one mana of any color\" and loses all other card types and abilities.\n\u{2212}9: If target player has fewer than nine poison counters, they get a number of poison counters equal to the difference.".to_string(),
        starting_loyalty: Some(6),
        abilities: vec![
            // 0: Draw a card and lose 1 life. Proliferate.
            // NOTE: Compleated (entering with 4 loyalty when life paid) not modeled in DSL.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(1),
                    },
                    Effect::Proliferate,
                ]),
                targets: vec![],
            },
            // -2: Target creature becomes a Treasure artifact.
            // TODO: No "permanent changes type/loses abilities" effect in DSL.
            // Effect::BecomeCopyOf or continuous type-modification layer effects
            // don't cover "becomes a Treasure artifact with specific activated ability".
            // -9: Poison counters equal to difference from 9.
            // TODO: No "poison counters equal to difference" EffectAmount variant.
            // Needs EffectAmount::PoisonDifference or similar.
        ],
        ..Default::default()
    }
}
