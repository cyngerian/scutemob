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
            // with `once_per_turn: true` and NO intervening-if (CR 603.2c/603.2h) -- the
            // per-(source, ability_index) `triggered_abilities_fired_this_turn` gate
            // (`abilities.rs`'s `flush_pending_triggers`), reset every untap step
            // (`layers.rs:1726-1744`), is an exact translation of "for the first time
            // each turn": her first attack of the turn queues, resolves, and marks the
            // ability fired; any later attack this same turn (any combat, any source of
            // the extra combat) is gated before it ever reaches the stack.
            //
            // PB-DX1 review Finding 1 (2026-08-01, closed on arrival -- OOS-DX1-5 is
            // filed as CLOSED, not open): this def previously used
            // `intervening_if: Some(Condition::IsFirstCombatPhase)` (`!turn.in_extra_combat`)
            // instead, which is a PROXY for "for the first time each turn", not a
            // translation, and PB-DX1's own fix made the divergence live as a
            // *suppressed* trigger on this `Complete`, deck-legal def -- the one
            // direction hard constraint 3 forbids: if Aurelia's first attack of the turn
            // happened in a LATER combat granted by another source (Aggravated Assault,
            // Moraug, World at War, Port Razer), `IsFirstCombatPhase` read false and
            // dropped the trigger the printed card requires to fire. `once_per_turn`
            // fires correctly in exactly that scenario, because it tracks Aurelia's own
            // attack history, not which combat phase of the turn this is. Karlach, Fury
            // of Avernus is the genuinely different case: her printed text says "if it's
            // the first combat phase of the turn" (controller-scoped `WheneverYouAttack`),
            // so `IsFirstCombatPhase` is a correct translation for her, not a proxy.
            AbilityDefinition::Triggered {
                once_per_turn: true,
                trigger_condition: TriggerCondition::WhenAttacks,
                intervening_if: None,
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
