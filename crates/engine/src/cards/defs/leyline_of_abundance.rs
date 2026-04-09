// Leyline of Abundance — {2}{G}{G}, Enchantment
// If this card is in your opening hand, you may begin the game with it on the battlefield.
// Whenever you tap a creature for mana, add an additional {G}.
// {6}{G}{G}: Put a +1/+1 counter on each creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("leyline-of-abundance"),
        name: "Leyline of Abundance".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 2, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "If Leyline of Abundance is in your opening hand, you may begin the game with it on the battlefield.\nWhenever you tap a creature for mana, add an additional {G}.\n{6}{G}{G}: Put a +1/+1 counter on each creature you control.".to_string(),
        abilities: vec![
            // TODO: DSL gap — "If in opening hand, begin game on battlefield" (Leyline).
            // CR 605.1b / CR 106.12a: "Whenever you tap a creature for mana, add {G}."
            // Triggered mana ability — resolves immediately (CR 605.4a).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenTappedForMana {
                    source_filter: ManaSourceFilter::Creature,
                },
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: ManaPool { green: 1, ..Default::default() },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 6, green: 2, ..Default::default() }),
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCreatureYouControl,
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
