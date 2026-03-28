// Thrasios, Triton Hero — {G}{U}, Legendary Creature — Merfolk Wizard 1/3
// {4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto
// the battlefield tapped. Otherwise, draw a card.
// Partner (You can have two commanders if both have partner.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thrasios-triton-hero"),
        name: "Thrasios, Triton Hero".to_string(),
        mana_cost: Some(ManaCost { green: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Merfolk", "Wizard"]),
        oracle_text: "{4}: Scry 1, then reveal the top card of your library. If it's a land card, put it onto the battlefield tapped. Otherwise, draw a card.\nPartner (You can have two commanders if both have partner.)".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Partner),
            // {4}: Scry 1, then reveal top card — land goes to battlefield tapped, else draw.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 4, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::Scry {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::RevealAndRoute {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            ..Default::default()
                        },
                        matched_dest: ZoneTarget::Battlefield { tapped: true },
                        unmatched_dest: ZoneTarget::Hand {
                            owner: PlayerTarget::Controller,
                        },
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
