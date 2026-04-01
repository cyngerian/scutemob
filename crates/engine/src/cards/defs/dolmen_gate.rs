// Dolmen Gate — {2}, Artifact
// Prevent all combat damage that would be dealt to attacking creatures you control.
//
// TODO: Effect::PreventAllCombatDamage prevents ALL combat damage (attacker and defender).
//   Oracle says "to attacking creatures you control" only — an attacker-only filter is a
//   DSL gap (no ReplacementModification::PreventCombatDamageTo(filter) for attacking creatures).
//   Using PreventAllCombatDamage as an overly broad approximation produces wrong game state
//   (also prevents damage to blockers). Per W5 policy, omitting until precise filter is available.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dolmen-gate"),
        name: "Dolmen Gate".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Prevent all combat damage that would be dealt to attacking creatures you control.".to_string(),
        abilities: vec![
            // TODO: No static replacement effect with filter "attacking creatures you control" exists.
            // Effect::PreventAllCombatDamage is too broad (prevents damage to all creatures).
            // DSL gap: ReplacementModification::PreventCombatDamageTo { filter: attacking_you_control }
            // needed. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
