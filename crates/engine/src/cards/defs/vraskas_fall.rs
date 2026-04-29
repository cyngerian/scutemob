// Vraska's Fall — {2}{B}, Instant
// Each opponent sacrifices a creature or planeswalker of their choice and gets a
// poison counter.
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
                // PB-SFT (CR 701.17a + CR 109.1c): OR-type filter via has_card_types.
                Effect::ForEach {
                    over: ForEachTarget::EachOpponent,
                    effect: Box::new(Effect::SacrificePermanents {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(1),
                        filter: Some(TargetFilter {
                            has_card_types: vec![CardType::Creature, CardType::Planeswalker],
                            ..Default::default()
                        }),
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
