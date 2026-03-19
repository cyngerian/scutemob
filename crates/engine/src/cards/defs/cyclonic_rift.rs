// Cyclonic Rift — {1}{U} Instant
// Return target nonland permanent you don't control to its owner's hand.
// Overload {6}{U} (You may cast this spell for its overload cost. If you do, change
// "target" in its text to "each.")
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cyclonic-rift"),
        name: "Cyclonic Rift".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text:
            "Return target nonland permanent you don't control to its owner's hand.\n\
             Overload {6}{U} (You may cast this spell for its overload cost. If you do, change \"target\" in its text to \"each.\")"
                .to_string(),
        abilities: vec![
            // CR 702.96a: Overload keyword marker.
            AbilityDefinition::Keyword(KeywordAbility::Overload),
            // CR 702.96a: Overload alternative cost {6}{U}.
            AbilityDefinition::Overload {
                cost: ManaCost { generic: 6, blue: 1, ..Default::default() },
            },
            // CR 702.96b: Spell effect — branches on WasOverloaded.
            // Normal: return one declared target nonland permanent to owner's hand.
            // Overloaded: return each nonland permanent opponents control to owner's hand.
            AbilityDefinition::Spell {
                effect: Effect::Conditional {
                    condition: Condition::WasOverloaded,
                    // Overloaded: bounce each nonland permanent opponents control to owner's hand
                    // (CR 702.96b, CR 108.3: "owner's hand" uses OwnerOf, not ControllerOf).
                    if_true: Box::new(Effect::ForEach {
                        over: ForEachTarget::EachPermanentMatching(TargetFilter {
                            non_land: true,
                            controller: TargetController::Opponent,
                            ..Default::default()
                        }),
                        effect: Box::new(Effect::MoveZone {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            to: ZoneTarget::Hand {
                                owner: PlayerTarget::OwnerOf(Box::new(
                                    EffectTarget::DeclaredTarget { index: 0 },
                                )),
                            },
                            controller_override: None,
                        }),
                    }),
                    // Normal cast: bounce the declared target to its owner's hand.
                    if_false: Box::new(Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::OwnerOf(Box::new(
                                EffectTarget::DeclaredTarget { index: 0 },
                            )),
                        },
                        controller_override: None,
                    }),
                },
                // Normal cast: target one nonland permanent you don't control.
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    non_land: true,
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
