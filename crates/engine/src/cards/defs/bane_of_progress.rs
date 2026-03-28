// Bane of Progress — {4}{G}{G} Creature — Elemental 2/2
// When this creature enters, destroy all artifacts and enchantments.
// Put a +1/+1 counter on this creature for each permanent destroyed this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bane-of-progress"),
        name: "Bane of Progress".to_string(),
        mana_cost: Some(ManaCost { generic: 4, green: 2, ..Default::default() }),
        types: creature_types(&["Elemental"]),
        oracle_text:
            "When Bane of Progress enters, destroy all artifacts and enchantments. Put a +1/+1 counter on this creature for each permanent destroyed this way."
                .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            // CR 701.8: Destroy all artifacts and enchantments (has_card_types: OR semantics).
            // AddCounterAmount uses EffectAmount::LastEffectCount set by the preceding DestroyAll.
            effect: Effect::Sequence(vec![
                Effect::DestroyAll {
                    filter: TargetFilter {
                        has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                        ..Default::default()
                    },
                    cant_be_regenerated: false,
                },
                Effect::AddCounterAmount {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: EffectAmount::LastEffectCount,
                },
            ]),
            intervening_if: None,
            targets: vec![],

            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
