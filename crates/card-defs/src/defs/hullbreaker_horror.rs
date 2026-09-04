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
//
// PB-DX35 (2026-09, OOS-DX4-2, re-adjudicated per execution-notes §0.5): NOT re-shaped into
// `mode_targets` here, on purpose -- moving these targets there would DROP the requirement
// rather than scope it, for a reason specific to this def. `AbilityDefinition::Keyword(Flash)`
// is `abilities[0]`, so this modal Triggered ability sits at REGISTRY index 1; but its
// `TriggerCondition::WheneverYouCastSpell` queues as a `PendingTriggerKind::Normal` trigger
// whose `ability_index` indexes the RUNTIME `characteristics.triggered_abilities` vec, where
// it is the only entry — index 0. The registry-based `ModeSelection` lookup
// (`rules::abilities::trigger_modal_plan`, which reads `modes` from the registry regardless
// of trigger kind — no runtime `TriggeredAbilityDef.modes` field exists) therefore looks up
// registry index 0, finds `Keyword(Flash)` there instead of this ability, and treats it as
// NOT modal. See the marker note below for the consequence. Filed as `OOS-DX35-1`.
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
        //
        // PB-DX35 (2026-09, OOS-DX4-2 re-adjudicated, execution-notes §0.5): the engine
        // change landed (`rules::abilities::trigger_modal_plan` now scopes a modal trigger's
        // target requirement to the CR 700.2b-legal chosen mode) but this def is NOT
        // re-shaped, and stays `partial` for the SAME observable defect through a DIFFERENT
        // mechanism: the modes lookup is registry-indexed and this ability sits at registry
        // index 1 (behind `Keyword(Flash)`), while its trigger queues with the RUNTIME index
        // 0 (see the module doc). The lookup therefore finds no `ModeSelection` at all,
        // treats the ability as non-modal, and both flat targets keep applying to the whole
        // trigger -- moving them into `mode_targets` here would silently DROP the requirement
        // (the trap `OOS-DX4-2`'s own row warns against) rather than scope it. Blast radius
        // is zero: this def is not deck-legal. Filed as `OOS-DX35-1`.
        completeness: Completeness::partial(
            "Modal triggered ability declares both mode targets flat. `trigger_modal_plan` \
             (PB-DX35) now scopes a modal trigger's targets to its CR 700.2b-legal chosen mode on \
             the trigger path, but this def's modal ability sits at REGISTRY index 1 (behind \
             Keyword(Flash)) while its trigger's RUNTIME ability_index is 0 -- the registry-based \
             ModeSelection lookup misses it entirely and treats the ability as non-modal, so both \
             mode targets still apply to the whole trigger. With no opponent spell on the stack \
             the whole trigger is dropped and the 'return target nonland permanent' mode stays \
             unreachable. See OOS-DX4-2 / OOS-DX35-1.",
        ),
        ..Default::default()
    }
}
