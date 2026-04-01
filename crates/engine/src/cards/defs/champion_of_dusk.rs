// Champion of Dusk — {3}{B}{B}, Creature — Vampire Knight 4/4
// When this enters, you draw X cards and you lose X life, where X is the number
// of Vampires you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("champion-of-dusk"),
        name: "Champion of Dusk".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Knight"]),
        oracle_text: "When this enters, you draw X cards and you lose X life, where X is the number of Vampires you control.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // CR 603.3: ETB trigger — draw X, lose X life where X = Vampires you control.
            // PermanentCount with Vampire subtype filter gives the count of Vampires you control,
            // including Champion of Dusk itself (which has just entered the battlefield).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::PermanentCount {
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                has_subtype: Some(SubType("Vampire".to_string())),
                                controller: TargetController::You,
                                ..Default::default()
                            },
                            controller: PlayerTarget::Controller,
                        },
                    },
                    Effect::LoseLife {
                        player: PlayerTarget::Controller,
                        amount: EffectAmount::PermanentCount {
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                has_subtype: Some(SubType("Vampire".to_string())),
                                controller: TargetController::You,
                                ..Default::default()
                            },
                            controller: PlayerTarget::Controller,
                        },
                    },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
