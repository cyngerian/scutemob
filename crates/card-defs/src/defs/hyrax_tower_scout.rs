// Hyrax Tower Scout — {2}{G}, Creature — Human Scout 3/3
// When this creature enters, untap target creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hyrax-tower-scout"),
        name: "Hyrax Tower Scout".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Scout"]),
        oracle_text: "When this creature enters, untap target creature.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenEntersBattlefield,
            effect: Effect::UntapPermanent {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            intervening_if: None,
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
