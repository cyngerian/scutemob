// Goblin Chirurgeon — {R}, Creature — Goblin Shaman 0/2
// Sacrifice a Goblin: Regenerate target creature.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-chirurgeon"),
        name: "Goblin Chirurgeon".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Shaman"]),
        oracle_text: "Sacrifice a Goblin: Regenerate target creature.".to_string(),
        power: Some(0),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Activated {
            // "Sacrifice a Goblin" — the source itself is a legal Goblin to sacrifice
            // (2004-10-04 ruling: "Can sacrifice itself"), so no exclude_self.
            cost: Cost::Sacrifice(TargetFilter {
                has_subtype: Some(SubType("Goblin".to_string())),
                ..Default::default()
            }),
            effect: Effect::Regenerate {
                target: EffectTarget::DeclaredTarget { index: 0 },
            },
            timing_restriction: None,
            targets: vec![TargetRequirement::TargetCreature],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        ..Default::default()
    }
}
