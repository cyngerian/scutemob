// Simic Ascendancy — {G}{U}, Enchantment
// {1}{G}{U}: Put a +1/+1 counter on target creature you control.
// Whenever one or more +1/+1 counters are put on a creature you control, put that many
// growth counters on this enchantment.
// At the beginning of your upkeep, if this enchantment has twenty or more growth counters
// on it, you win the game.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("simic-ascendancy"),
        name: "Simic Ascendancy".to_string(),
        mana_cost: Some(ManaCost { green: 1, blue: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "{1}{G}{U}: Put a +1/+1 counter on target creature you control.\nWhenever one or more +1/+1 counters are put on a creature you control, put that many growth counters on Simic Ascendancy.\nAt the beginning of your upkeep, if Simic Ascendancy has twenty or more growth counters on it, you win the game.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, green: 1, blue: 1, ..Default::default() }),
                effect: Effect::AddCounter {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // ENGINE-BLOCKED: "Whenever one or more +1/+1 counters are put on a creature you
            // control, put that many growth counters on this enchantment." PB-AC1 shipped
            // `TriggerCondition::WhenCounterPlaced { on_self: false, filter: creature you
            // control }`, which covers the TRIGGER half. The EFFECT half is still blocked:
            // "that many" requires an `EffectAmount` that reads the count of counters from the
            // triggering event (e.g. a Master Biomancer-style multi-counter placement), and no
            // such `EffectAmount` variant exists — only `EffectAmount::Fixed`/`XValue`/etc.
            // Authoring with `Fixed(1)` would be wrong whenever 2+ counters land at once.
            // ENGINE-BLOCKED: "At the beginning of your upkeep, if this enchantment has twenty
            // or more growth counters on it, you win the game." — win-the-game effect gated on
            // a counter-count condition is a separate primitive (PB-AC8).
        ],
        ..Default::default()
    }
}
