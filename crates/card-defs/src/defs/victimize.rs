// Victimize — {2}{B}, Sorcery
// "Choose two target creature cards in your graveyard. Sacrifice a creature.
//  If you do, return the chosen cards to the battlefield tapped."
//
// Victimize ruling (2020-11-10): the sacrifice is mandatory-if-able — you must
// sacrifice a creature if able as Victimize resolves, and can't decline. If both
// target cards are illegal, Victimize won't resolve (no sacrifice happens). If one
// target is illegal, the sacrifice still happens and the other card returns.
//
// CR 608.2c/608.2h (PB-EF10): "Sacrifice a creature. If you do, ..." is gated by
// Condition::SacrificeFired — true iff the SacrificePermanents immediately before
// it actually moved >= 1 permanent (i.e. the controller controlled a creature to
// sacrifice). Each MoveZone independently no-ops on an already-illegal target
// (CR 608.2b), so one illegal target still returns the other.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("victimize"),
        name: "Victimize".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose two target creature cards in your graveyard. Sacrifice a creature. \
                      If you do, return the chosen cards to the battlefield tapped."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // "Sacrifice a creature." -- mandatory-if-able (Victimize ruling).
                Effect::SacrificePermanents {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                },
                // "If you do, return the chosen cards to the battlefield tapped."
                Effect::Conditional {
                    condition: Condition::SacrificeFired,
                    if_true: Box::new(Effect::Sequence(vec![
                        Effect::MoveZone {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            to: ZoneTarget::Battlefield { tapped: true },
                            controller_override: Some(PlayerTarget::Controller),
                        },
                        Effect::MoveZone {
                            target: EffectTarget::DeclaredTarget { index: 1 },
                            to: ZoneTarget::Battlefield { tapped: true },
                            controller_override: Some(PlayerTarget::Controller),
                        },
                    ])),
                    if_false: Box::new(Effect::Nothing),
                },
            ]),
            targets: vec![
                TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
