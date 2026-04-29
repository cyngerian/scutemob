// Vraska's Fall — {2}{B}, Instant
// Each opponent sacrifices a creature or planeswalker of their choice and gets a
// poison counter.
//
// The "creature or planeswalker" choice in SacrificePermanents covers both types.
// Poison counter via ForEach over EachOpponent.
//
// Note: SacrificePermanents does not filter creature-or-planeswalker specifically;
// it picks the player's lowest-ID permanent. This is a DSL gap (W5-acceptable partial
// since the sacrifice type filter is available via PB-SFT, but the poison counter
// Effect::AddCounter does not support Player targets — it is a silent no-op when
// target resolves to a player). Keeping filter: None until poison-counter-to-player
// is properly handled.
// TODO: SacrificePermanents filter is available (PB-SFT), but Effect::AddCounter to a
// Player target is a silent no-op (resolver only handles ResolvedTarget::Object). This
// card remains blocked on AddCounter-to-Player support.
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
                // TODO: SacrificePermanents filter is available (PB-SFT), but the poison
                // counter half (Effect::AddCounter to a Player target) is a silent no-op.
                // Keeping filter: None until AddCounter-to-Player is supported.
                Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::SacrificePermanents {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(1),
                        filter: None,
                    }),
                },
                // "Each opponent gets a poison counter."
                // TODO: Effect::AddCounter only handles ResolvedTarget::Object; targeting a
                // Player (DeclaredTarget index 0 in ForEach context resolves to a player) is
                // a silent no-op. This half of the card does nothing until AddCounter-to-Player
                // is implemented.
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
