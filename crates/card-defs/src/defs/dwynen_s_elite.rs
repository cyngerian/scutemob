// Dwynen's Elite — {1}{G}, Creature — Elf Warrior 2/2
// When this creature enters, if you control another Elf, create a 1/1 green Elf Warrior
// creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dwynen-s-elite"),
        name: "Dwynen's Elite".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Elf", "Warrior"]),
        oracle_text: "When this creature enters, if you control another Elf, create a 1/1 green \
                      Elf Warrior creature token."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "If you control another Elf" intervening-if — Condition lacks
            // "you control a permanent with subtype X" variant. Implementing trigger
            // without the condition would create a token even without another Elf,
            // which is wrong behavior. Using TODO per W5 policy.
        ],
        completeness: Completeness::inert(
            "Blocked on the 'another' exclusion, not on subtype counting. 'When this creature \
             enters, if you control ANOTHER Elf' — Condition::ControlCreatureWithSubtype \
             (card_definition.rs:3536) and Condition::YouControlNOrMoreWithFilter (:3571) both \
             exist, but neither can exclude the source: ControlCreatureWithSubtype has no \
             exclusion, and YouControlNOrMoreWithFilter's evaluator (effects/mod.rs:8508-8536) \
             silently ignores TargetFilter.exclude_self. Implementing without the exclusion would \
             create a token off Dwynen's Elite alone, which is wrong. Omitted per W5 policy.",
        ),
        ..Default::default()
    }
}
