// Cryptic Command — {1}{U}{U}{U}, Instant
// Choose two —
// • Counter target spell.
// • Return target permanent to its owner's hand.
// • Tap all creatures your opponents control.
// • Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cryptic-command"),
        name: "Cryptic Command".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 3, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose two —\n• Counter target spell.\n• Return target permanent to its owner's hand.\n• Tap all creatures your opponents control.\n• Draw a card.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — targets are declared only for
            // the two chosen modes. `Spell.targets` is empty; each mode's requirements
            // live in `mode_targets`, and effects use LOCAL (0-based) DeclaredTarget
            // indices. Previously this card was a stubbed `Effect::Nothing` (all four
            // sub-effects exist in the DSL but per-mode targeting did not).
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 2,
                max_modes: 2,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Counter target spell.
                    Effect::CounterSpell {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    // Mode 1: Return target permanent to its owner's hand.
                    // CR 108.3: "owner's hand" uses OwnerOf, not ControllerOf.
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget {
                                index: 0,
                            })),
                        },
                        controller_override: None,
                    },
                    // Mode 2: Tap all creatures your opponents control. No target.
                    Effect::TapPermanent {
                        target: EffectTarget::AllPermanentsMatching(Box::new(TargetFilter {
                            has_card_type: Some(CardType::Creature),
                            controller: TargetController::Opponent,
                            ..Default::default()
                        })),
                    },
                    // Mode 3: Draw a card. No target.
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                ],
                mode_targets: Some(vec![
                    vec![TargetRequirement::TargetSpell],
                    vec![TargetRequirement::TargetPermanent],
                    vec![],
                    vec![],
                ]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
