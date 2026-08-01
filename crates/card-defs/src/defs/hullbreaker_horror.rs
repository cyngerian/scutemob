// Hullbreaker Horror — {5}{U}{U}, Creature — Kraken Horror 7/8
// Flash
// This spell can't be countered.
// Whenever you cast a spell, choose up to one —
// • Return target spell you don't control to its owner's hand.
// • Return target nonland permanent to its owner's hand.
//
// CR 700.2b / PB-35: Modal triggered ability with "choose up to one" (min_modes: 0).
// Bot fallback: mode 0 (bounce opponent's spell) when target exists, else 0 modes.
// CR 101.6: "This spell can't be countered" — CardDefinition.cant_be_countered = true.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hullbreaker-horror"),
        name: "Hullbreaker Horror".to_string(),
        mana_cost: Some(ManaCost {
            generic: 5,
            blue: 2,
            ..Default::default()
        }),
        types: creature_types(&["Kraken", "Horror"]),
        oracle_text: "Flash\nThis spell can't be countered.\nWhenever you cast a spell, choose up \
                      to one —\n• Return target spell you don't control to its owner's hand.\n• \
                      Return target nonland permanent to its owner's hand."
            .to_string(),
        power: Some(7),
        toughness: Some(8),
        cant_be_countered: true,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // CR 700.2b / PB-35: "Whenever you cast a spell, choose up to one" modal trigger.
            // min_modes: 0 = "up to one" (may choose zero modes).
            // Bot: auto-selects mode 0 (bounce opponent's spell). If no legal target, 0 modes.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    spell_type_filter: None,
                    noncreature_only: false,
                    chosen_subtype_filter: false,
                    during_opponent_turn: false,
                    spell_subtype_filter: None,
                },
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![
                    // Mode 0 target: a spell you don't control (on the stack).
                    TargetRequirement::TargetSpellWithFilter(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    }),
                    // Mode 1 target: a nonland permanent.
                    TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        non_land: true,
                        ..Default::default()
                    }),
                ],
                modes: Some(ModeSelection {
                    min_modes: 0, // "choose up to one" — may choose zero modes
                    max_modes: 1,
                    modes: vec![
                        // Mode 0: Return target spell you don't control to its owner's hand.
                        Effect::MoveZone {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            to: ZoneTarget::Hand {
                                owner: PlayerTarget::OwnerOf(Box::new(
                                    EffectTarget::DeclaredTarget { index: 0 },
                                )),
                            },
                            controller_override: None,
                        },
                        // Mode 1: Return target nonland permanent to its owner's hand.
                        Effect::MoveZone {
                            target: EffectTarget::DeclaredTarget { index: 1 },
                            to: ZoneTarget::Hand {
                                owner: PlayerTarget::OwnerOf(Box::new(
                                    EffectTarget::DeclaredTarget { index: 1 },
                                )),
                            },
                            controller_override: None,
                        },
                    ],
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    mode_targets: None,
                }),
                trigger_zone: None,
            },
        ],
        // PB-DX4 fix cycle (2026-08-01, `scutemob-168`, review Finding 2):
        // Complete (by the `#[default]` derive) -> partial.
        //
        // MCP-verified printed text: "Whenever you cast a spell, choose **up to one** —
        // • Return target spell you don't control to its owner's hand. • Return target
        // nonland permanent to its owner's hand."
        //
        // Both mode targets are declared FLAT on the trigger with `mode_targets: None`, so
        // both are required whichever mode is chosen — and `rules/abilities.rs`'s CR 603.3d
        // auto-target path skips a trigger outright when **any** required slot has no legal
        // candidate ("If any required target has no legal candidate, skip this trigger").
        // "Target spell you don't control" is unsatisfiable in the ordinary case: unless an
        // opponent has a spell on the stack at the moment you cast yours, the WHOLE trigger
        // is dropped, so the second mode — an unconditional bounce that is the card's main
        // use — is unreachable too, and `min_modes: 0`'s "up to one" never gets to mean
        // anything.
        //
        // This is the identical defect `shambling_ghast` was demoted for in the same batch,
        // and it is dispositioned the same way rather than merely named: PB-DX4's original
        // pass classified this def class-B and recorded the shape only in seed OOS-DX4-2,
        // which is a description, not a marker. Closing it needs `mode_targets` honoured on
        // the triggered-ability path (today every consumer is on the casting path), so it is
        // an engine change and not a card-def edit.
        completeness: Completeness::partial(
            "Modal triggered ability declares both mode targets flat (ModeSelection.mode_targets \
             is honoured only on the casting path, never for triggered abilities), so 'target \
             spell you don't control' is required for BOTH modes. rules/abilities.rs skips a \
             trigger when any required slot has no legal candidate, so with no opponent spell on \
             the stack the whole trigger is dropped and the 'return target nonland permanent' \
             mode is unreachable. See OOS-DX4-2.",
        ),
        ..Default::default()
    }
}
