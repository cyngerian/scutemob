// 63. Kitchen Finks — {1}{G/W}{G/W}, Creature — Ouphe 3/2; ETB gain 2 life. Persist.
// Oracle cost is {1}{G/W}{G/W} (hybrid); simplified here to {1}{W}{G} because
// the ManaCost struct does not support hybrid mana symbols.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kitchen-finks"),
        name: "Kitchen Finks".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Ouphe"]),
        oracle_text: "When this creature enters, you gain 2 life.\nPersist (When this creature dies, if it had no -1/-1 counters on it, return it to the battlefield under its owner's control with a -1/-1 counter on it.)".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Persist),
        ],
    }
}
