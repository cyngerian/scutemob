// Call of the Ring — {3}{B}, Enchantment
// "When Call of the Ring enters the battlefield, the Ring tempts you."
// "At the beginning of your upkeep, you lose 1 life for each burden counter on Call of the Ring,
//  then put a burden counter on Call of the Ring."
//
// CR 701.54a: "The Ring tempts you" is a keyword action — advance ring level, choose ring-bearer.
// CR 603.2: ETB triggered ability fires when Call of the Ring enters the battlefield.
// CR 702.24 (by analogy): Burden counter is a named counter unique to this card.
//
// DSL gap: "you lose 1 life for each burden counter on this permanent" requires
// EffectAmount::CountersOnSelf(CounterType::Custom("Burden")) — not yet in the DSL.
// The life-loss portion of the upkeep trigger is deferred; only the counter placement
// is implemented. Tracked alongside Canopy Crawler (same EffectAmount::CountersOnSelf gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("call-of-the-ring"),
        name: "Call of the Ring".to_string(),
        mana_cost: Some(ManaCost { black: 1, generic: 3, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "When Call of the Ring enters the battlefield, the Ring tempts you.\nAt the beginning of your upkeep, you lose 1 life for each burden counter on Call of the Ring, then put a burden counter on Call of the Ring.".to_string(),
        abilities: vec![
            // CR 701.54a / CR 603.2: ETB — the Ring tempts you when Call of the Ring enters.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::TheRingTemptsYou,
                intervening_if: None,
            },
            // CR 603.2 / CR 122: At the beginning of your upkeep, put a burden counter on this.
            // TODO: Also "you lose 1 life for each burden counter on Call of the Ring" — requires
            // EffectAmount::CountersOnSelf(CounterType::Custom("Burden")) which is not yet in the
            // DSL. Life-loss portion deferred until that EffectAmount variant is added.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Custom("Burden".to_string()),
                    count: 1,
                },
                intervening_if: None,
            },
        ],
        ..Default::default()
    }
}
