// Miara, Thorn of the Glade — {1}{B}, Legendary Creature — Elf Scout 1/2
// Whenever Miara or another Elf you control dies, you may pay {1} and 1 life.
// If you do, draw a card.
// Partner
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("miara-thorn-of-the-glade"),
        name: "Miara, Thorn of the Glade".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Scout"],
        ),
        oracle_text: "Whenever Miara, Thorn of the Glade or another Elf you control dies, you may pay {1} and 1 life. If you do, draw a card.\nPartner (You can have two commanders if both have partner.)".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Partner),
            // PB-AC2 (CR 118.12): "Whenever Miara or another Elf you control dies, you may
            // pay {1} and 1 life. If you do, draw a card." The Elf filter matches Miara
            // herself (she is an Elf) so exclude_self: false correctly covers "Miara or
            // another Elf".
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Elf".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Sequence(vec![
                        Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                        Cost::PayLife(1),
                    ]),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
