// Incendiary Command — {3}{R}{R}, Sorcery
// Choose two —
// • Incendiary Command deals 4 damage to target player or planeswalker.
// • Incendiary Command deals 2 damage to each creature.
// • Destroy target nonbasic land.
// • Each player discards all the cards in their hand, then draws that many cards.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("incendiary-command"),
        name: "Incendiary Command".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Choose two —\n• Incendiary Command deals 4 damage to target player or planeswalker.\n• Incendiary Command deals 2 damage to each creature.\n• Destroy target nonbasic land.\n• Each player discards all the cards in their hand, then draws that many cards.".to_string(),
        abilities: vec![
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — modes 0, 1, 2 are fully
            // expressible and migrated. `Spell.targets` is empty; each mode's target (if
            // any) lives in `mode_targets` at LOCAL index 0.
            //
            // ENGINE-BLOCKED (mode 3 only): "each player discards all the cards in their
            // hand, then draws that many cards" (a wheel effect). `EffectAmount::HandSize`
            // exists (PB-AC3) but is evaluated dynamically at each effect's execution time
            // within a `Sequence` — a `DiscardCards { count: HandSize }` followed by
            // `DrawCards { count: HandSize }` would discard the whole hand, THEN read a
            // hand size of 0 for the draw (no snapshot-before-discard mechanism exists;
            // `EffectAmount::LastEffectCount` only tracks DestroyAll/ExileAll/bounce/untap
            // counts, not DiscardCards). Implementing this mode would silently draw 0 cards
            // instead of "that many" — a wrong-game-state bug, so it is left as a
            // documented no-op instead. Unrelated to AC4's per-mode-targeting scope.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: Some(ModeSelection {
                    min_modes: 2,
                    max_modes: 2,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: 4 damage to target player or planeswalker.
                        Effect::DealDamage {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(4),
                        },
                        // Mode 1: 2 damage to each creature. No target.
                        Effect::DealDamage {
                            target: EffectTarget::AllCreatures,
                            amount: EffectAmount::Fixed(2),
                        },
                        // Mode 2: Destroy target nonbasic land.
                        Effect::DestroyPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                            cant_be_regenerated: false,
                        },
                        // Mode 3: Each player discards all cards in hand, then draws that
                        // many. See file-level ENGINE-BLOCKED comment above.
                        Effect::Nothing,
                    ],
                    mode_targets: Some(vec![
                        vec![TargetRequirement::TargetPlayerOrPlaneswalker],
                        vec![],
                        vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                            has_card_type: Some(CardType::Land),
                            nonbasic: true,
                            ..Default::default()
                        })],
                        vec![],
                    ]),
                }),
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
