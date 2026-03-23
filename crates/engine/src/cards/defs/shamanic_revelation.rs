// Shamanic Revelation — {3}{G}{G}, Sorcery
// Draw a card for each creature you control.
// Ferocious — You gain 4 life for each creature you control with power 4 or greater.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shamanic-revelation"),
        name: "Shamanic Revelation".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw a card for each creature you control.\nFerocious — You gain 4 life for each creature you control with power 4 or greater.".to_string(),
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
                // TODO: Ferocious — gain 4 life for each creature with power 4+.
                //   EffectAmount lacks a "count permanents matching power filter" variant.
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
