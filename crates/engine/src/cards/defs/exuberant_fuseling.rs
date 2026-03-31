// Exuberant Fuseling — {R}, Creature — Phyrexian Goblin Warrior 0/1
// Trample
// This creature gets +1/+0 for each oil counter on it.
// When this creature enters and whenever another creature or artifact you control
// is put into a graveyard from the battlefield, put an oil counter on this creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("exuberant-fuseling"),
        name: "Exuberant Fuseling".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Goblin", "Warrior"]),
        oracle_text: "Trample\nThis creature gets +1/+0 for each oil counter on it.\nWhen this creature enters and whenever another creature or artifact you control is put into a graveyard from the battlefield, put an oil counter on this creature.".to_string(),
        power: Some(0),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: "This creature gets +1/+0 for each oil counter on it" — CDA requiring
            // EffectFilter::Self + dynamic count based on counters. DSL lacks
            // ModifyPower(EffectAmount::CounterCountOnSelf) or equivalent CDA expression.
            // Static ability omitted per W5 policy.

            // ETB: put an oil counter on this creature.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::Oil,
                    count: 1,
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // TODO: "whenever another creature or artifact you control is put into a graveyard
            // from the battlefield, put an oil counter on this creature" —
            // WheneverCreatureDies covers the creature half (with exclude_self=true, controller=You)
            // but there is no "or artifact" variant in WheneverCreatureDies, and no separate
            // WheneverArtifactDies trigger condition exists. Implementing only the creature
            // half would produce wrong game state (misses artifact deaths). Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
