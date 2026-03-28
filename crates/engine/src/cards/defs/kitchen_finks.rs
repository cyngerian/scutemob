// 63. Kitchen Finks — {1}{G/W}{G/W}, Creature — Ouphe 3/2; ETB gain 2 life. Persist.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kitchen-finks"),
        name: "Kitchen Finks".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            hybrid: vec![
                HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
                HybridMana::ColorColor(ManaColor::Green, ManaColor::White),
            ],
            ..Default::default()
        }),
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
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            AbilityDefinition::Keyword(KeywordAbility::Persist),
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
    }
}
