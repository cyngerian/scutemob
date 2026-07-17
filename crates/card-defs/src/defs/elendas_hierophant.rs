// Elenda's Hierophant — {2}{W}, Creature — Vampire Cleric 1/1
// Flying
// Whenever you gain life, put a +1/+1 counter on this creature.
// When this creature dies, create X 1/1 white Vampire creature tokens with lifelink,
// where X is its power.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elendas-hierophant"),
        name: "Elenda's Hierophant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Cleric"]),
        oracle_text: "Flying\nWhenever you gain life, put a +1/+1 counter on this creature.\nWhen \
                      this creature dies, create X 1/1 white Vampire creature tokens with \
                      lifelink, where X is its power."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Whenever you gain life, put a +1/+1 counter on this creature.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverYouGainLife,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "When dies, create X Vampires where X = power" — power-based token count
            // not in DSL
        ],
        completeness: Completeness::partial(
            "Add the death trigger: TriggerCondition::WhenDies + Effect::CreateToken with a 1/1 \
             white Vampire TokenSpec carrying KeywordAbility::Lifelink and count: \
             EffectAmount::SourcePowerAtLastKnownInformation (effects/mod.rs:7003; lki_power \
             captured at abilities.rs:4167). Mirror elenda_the_dusk_rose.rs's TokenSpec, but use \
             the LKI power amount rather than a fixed count.",
        ),
        ..Default::default()
    }
}
