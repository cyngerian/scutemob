// Aurelia, the Warleader — {2}{R}{R}{W}{W}, Legendary Creature — Angel 3/4
// Flying, vigilance, haste
// Whenever Aurelia attacks for the first time each turn, untap all creatures you control.
// After this phase, there is an additional combat phase.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("aurelia-the-warleader"),
        name: "Aurelia, the Warleader".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            white: 2,
            ..Default::default()
        }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Angel"]),
        oracle_text: "Flying, vigilance, haste\nWhenever Aurelia attacks for the first time each \
                      turn, untap all creatures you control. After this phase, there is an \
                      additional combat phase."
            .to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // "Whenever Aurelia attacks for the first time each turn" maps to WhenAttacks
            // with Condition::IsFirstCombatPhase (Karlach, Fury of Avernus uses the same
            // IsFirstCombatPhase intervening-if, but on WheneverYouAttack instead of
            // WhenAttacks -- her printed text is controller-scoped, "whenever you attack",
            // not self-scoped like Aurelia's).
            //
            // OOS-DX1-5 (PB-DX1, 2026-08-01): `IsFirstCombatPhase` is a PROXY for "for the
            // first time each turn", not a translation, and the two diverge. The printed
            // card triggers the first time Aurelia herself attacks in a turn, however late;
            // `IsFirstCombatPhase` instead asks "is this the turn's first combat phase at
            // all" (`!state.turn.in_extra_combat`). They agree whenever Aurelia's first
            // attack of the turn happens to be in the turn's first combat -- the overwhelming
            // common case -- but diverge if she is blinked in (or otherwise made available to
            // attack) only during a LATER combat phase granted by another source: the real
            // card would still trigger on that first attack of hers, but this def's
            // `IsFirstCombatPhase` reads false (already in an extra combat) and suppresses
            // it. The faithful authoring is `once_per_turn: true` with no `intervening_if`
            // (expressible since PB-DX1 §10 propagates `once_per_turn` through the lowering)
            // -- deliberately NOT done here: re-authoring would change which mechanism T1
            // (`test_dx1_aurelia_attack_trigger_fires_exactly_once_per_turn`,
            // `crates/engine/tests/primitives/pb_dx1_lowered_intervening_if.rs`) exercises,
            // and the substitution needs to be argued on its own oracle merits in a
            // dedicated pass, not folded into the batch that made it possible.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                intervening_if: Some(Condition::IsFirstCombatPhase),
                effect: Effect::Sequence(vec![
                    // Untap all creatures you control.
                    Effect::ForEach {
                        over: ForEachTarget::EachCreatureYouControl,
                        effect: Box::new(Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                    // After this phase, there is an additional combat phase.
                    Effect::AdditionalCombatPhase {
                        followed_by_main: false,
                    },
                ]),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
