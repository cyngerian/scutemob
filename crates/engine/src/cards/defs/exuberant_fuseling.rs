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
            // TODO: "This creature gets +1/+0 for each oil counter on it." — CR 613.4c Layer 7c
            // modify (CR 611.3a: static ability, NOT locked-in; must re-evaluate continuously).
            //
            // Blocked by: dynamic-static Layer-7c modify primitive (PB-CC-C-followup or similar).
            // PB-CC-C added LayerModification::ModifyPowerDynamic and ModifyToughnessDynamic for
            // one-shot spell use cases (CR 608.2h substitution-at-resolution, e.g. Olivia's
            // Wrath pattern). Fuseling needs CR 611.3a continuous re-evaluation, which requires
            // a separate `AbilityDefinition::CdaModifyPowerToughness` variant analogous to
            // `CdaPowerToughness` (Layer 7a). Routing this through ApplyContinuousEffect would
            // produce a stale-snapshot bug: the oil counter count freezes at ETB time, so
            // subsequent counter changes do not update power, violating oracle text.
            // See `memory/primitives/pb-review-CC-C.md` C1/E4 for full analysis.

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
