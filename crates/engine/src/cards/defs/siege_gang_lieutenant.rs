// Siege-Gang Lieutenant — {3}{R}, Creature — Goblin 2/2
// Lieutenant — At the beginning of combat on your turn, if you control your commander,
//   create two 1/1 red Goblin creature tokens. Those tokens gain haste until end of turn.
// {2}, Sacrifice a Goblin: This creature deals 1 damage to any target.
//
// NOTE: Lieutenant ability word (CR 702.124 is not Lieutenant; CR uses "Lieutenant" as an
// ability word) requires "if you control your commander" as an intervening-if condition
// (Condition::YouControlYourCommander not in DSL). The token creation + haste grant are
// implementable individually but the commander-control intervening-if check has no DSL
// expression. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("siege-gang-lieutenant"),
        name: "Siege-Gang Lieutenant".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Lieutenant \u{2014} At the beginning of combat on your turn, if you control your commander, create two 1/1 red Goblin creature tokens. Those tokens gain haste until end of turn.\n{2}, Sacrifice a Goblin: This creature deals 1 damage to any target.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: Lieutenant — "At the beginning of combat on your turn, if you control your
            // commander, create two 1/1 red Goblin creature tokens that gain haste until EOT."
            // Requires Condition::YouControlYourCommander as intervening-if on
            // AtBeginningOfCombat trigger. That condition is not in the DSL. Omitted per W5.

            // {2}, Sacrifice a Goblin: This creature deals 1 damage to any target.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 2, ..Default::default() }),
                    Cost::Sacrifice(TargetFilter {
                        has_subtype: Some(SubType("Goblin".to_string())),
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetAny],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
