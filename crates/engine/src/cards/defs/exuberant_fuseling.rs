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
            // "This creature gets +1/+0 for each oil counter on it."
            // CR 611.3a: static ability, not locked-in — continuously re-evaluates.
            // CR 613.4c: Layer 7c modify (power only; toughness unchanged).
            // CounterCount with EffectTarget::Source counts oil counters on this creature.
            // PB-CC-C-followup ships AbilityDefinition::CdaModifyPowerToughness to
            // register a ContinuousEffect with ModifyPowerDynamic + is_cda=true at Layer 7c.
            AbilityDefinition::CdaModifyPowerToughness {
                power: Some(EffectAmount::CounterCount {
                    target: EffectTarget::Source,
                    counter: CounterType::Oil,
                }),
                toughness: None,
            },

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
            // half would produce wrong game state (misses artifact deaths).
            // Blocked by: WheneverCreatureOrArtifactDies trigger condition (multi-blocker).
            // Out of PB-CC-C scope per memory/primitives/pb-retriage-CC.md.
        ],
        ..Default::default()
    }
}
