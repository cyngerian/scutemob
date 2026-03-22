// Incubation Druid — {1}{G}, Creature — Elf Druid 0/2
// {T}: Add one mana of any type that a land you control could produce. If this creature has a
// +1/+1 counter on it, add three mana of that type instead.
// {3}{G}{G}: Adapt 3. (If this creature has no +1/+1 counters on it, put three +1/+1 counters on it.)
//
// TODO: First ability — two DSL gaps:
//   (1) "mana of any type that a land you control could produce" requires querying which colors
//       your lands can produce at runtime; no Effect variant for this exists.
//   (2) "if this creature has a +1/+1 counter, add three mana instead" requires a Conditional
//       that branches between AddManaAnyColor (×1) and a triple-mana variant (×3) based on
//       SourceHasCounters. Neither a triple-add variant nor the land-type restriction exists.
// Implementing only the Adapt 3 activated ability; mana tap ability left unimplemented
// per W5 policy (wrong game state is worse than empty).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("incubation-druid"),
        name: "Incubation Druid".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "{T}: Add one mana of any type that a land you control could produce. If this creature has a +1/+1 counter on it, add three mana of that type instead.\n{3}{G}{G}: Adapt 3. (If this creature has no +1/+1 counters on it, put three +1/+1 counters on it.)".to_string(),
        power: Some(0),
        toughness: Some(2),
        abilities: vec![
            // Keyword marker for Adapt 3 (CR 701.46)
            AbilityDefinition::Keyword(KeywordAbility::Adapt(3)),
            // Activated ability: {3}{G}{G}, {T}: Adapt 3
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, green: 2, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::Conditional {
                    condition: Condition::SourceHasNoCountersOfType {
                        counter: CounterType::PlusOnePlusOne,
                    },
                    if_true: Box::new(Effect::AddCounter {
                        target: EffectTarget::Source,
                        counter: CounterType::PlusOnePlusOne,
                        count: 3,
                    }),
                    if_false: Box::new(Effect::Nothing),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
