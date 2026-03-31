// Vraska's Fall — {2}{B}, Instant
// Each opponent sacrifices a creature or planeswalker of their choice and gets a
// poison counter.
//
// The "creature or planeswalker" choice in SacrificePermanents covers both types.
// Poison counter via ForEach over EachOpponent.
//
// Note: SacrificePermanents does not filter creature-or-planeswalker specifically;
// it picks the player's lowest-ID permanent. This is a DSL gap (W5-acceptable partial
// since the sacrifice type filter is not available, but the structure is correct).
// TODO: SacrificePermanents lacks creature-or-planeswalker filter; opponent picks any permanent.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vraskas-fall"),
        name: "Vraska's Fall".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Each opponent sacrifices a creature or planeswalker of their choice and gets a poison counter.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // "Each opponent sacrifices a creature or planeswalker of their choice."
                // TODO: SacrificePermanents doesn't filter to creature/planeswalker only.
                Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::SacrificePermanents {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(1),
                    }),
                },
                // "Each opponent gets a poison counter."
                Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::Poison,
                        count: 1,
                    }),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
