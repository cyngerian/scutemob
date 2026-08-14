// Frantic Search — {2}{U}, Instant
// Draw two cards, then discard two cards. Untap up to three lands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("frantic-search"),
        name: "Frantic Search".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Draw two cards, then discard two cards. Untap up to three lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 115.10 (PB-DX28): "untap up to three lands" is printed with no
            // "target" — a resolution-time UNTARGETED choice, not a declared target.
            effect: Effect::Sequence(vec![
                Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                Effect::DiscardCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(2),
                },
                Effect::UntapPermanent {
                    target: EffectTarget::ChosenObject {
                        zone: ChoiceZone::Battlefield,
                        filter: Box::new(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        }),
                        count: 3,
                        up_to: true,
                    },
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
