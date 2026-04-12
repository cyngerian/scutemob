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
        oracle_text: "Whenever Miara, Thorn of the Glade or another Elf you control dies, you may pay {1} and 1 life. If you do, draw a card.\nPartner".to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Elf you control dies" + "may pay {1} and 1 life" — WheneverCreatureDies
            //   is overbroad + optional pay-to-draw not expressible. Implementing as
            //   unconditional draw on creature death (approximation).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureDies { controller: Some(TargetController::You), exclude_self: false, nontoken_only: false, filter: None,
},
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Partner),
        ],
        ..Default::default()
    }
}
