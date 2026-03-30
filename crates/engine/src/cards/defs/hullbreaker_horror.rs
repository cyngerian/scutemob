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
        mana_cost: Some(ManaCost { generic: 5, blue: 2, ..Default::default() }),
        types: creature_types(&["Kraken", "Horror"]),
        oracle_text: "Flash\nThis spell can't be countered.\nWhenever you cast a spell, choose up to one —\n• Return target spell you don't control to its owner's hand.\n• Return target nonland permanent to its owner's hand.".to_string(),
        power: Some(7),
        toughness: Some(8),
        cant_be_countered: true,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            // CR 700.2b / PB-35: "Whenever you cast a spell, choose up to one" modal trigger.
            // min_modes: 0 = "up to one" (may choose zero modes).
            // Bot: auto-selects mode 0 (bounce opponent's spell). If no legal target, 0 modes.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    spell_type_filter: None,
                    noncreature_only: false,
                    during_opponent_turn: false,
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
                }),
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
