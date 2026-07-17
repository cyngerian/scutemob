// Shamanic Revelation — {3}{G}{G}, Sorcery
// Draw a card for each creature you control.
// Ferocious — You gain 4 life for each creature you control with power 4 or greater.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shamanic-revelation"),
        name: "Shamanic Revelation".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            green: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw a card for each creature you control.\nFerocious — You gain 4 life for \
                      each creature you control with power 4 or greater."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // Draw a card for each creature you control.
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::PermanentCount {
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            controller: TargetController::You,
                            ..Default::default()
                        },
                        controller: PlayerTarget::Controller,
                    },
                },
                // Ferocious — gain 4 life for each creature you control with power 4+.
                // ForEach + fixed-4 GainLife per matching creature sums to "4 life for each".
                Effect::ForEach {
                    over: ForEachTarget::EachPermanentMatching(Box::new(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        controller: TargetController::You,
                        min_power: Some(4),
                        ..Default::default()
                    })),
                    effect: Box::new(Effect::GainLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::Fixed(4),
                    }),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
