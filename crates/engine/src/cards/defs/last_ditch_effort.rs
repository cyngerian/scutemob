// Last-Ditch Effort — {R}, Instant
// Sacrifice any number of creatures. Last-Ditch Effort deals that much damage to any target.
//
// TODO: "Sacrifice any number of creatures" — no Cost::SacrificeAnyNumber or
//   Effect::SacrificeAnyNumberFor { filter: creature } in DSL.
//   The damage amount equals the number of creatures sacrificed, which requires tracking
//   a variable cost choice. DSL gap: need SacrificeAnyNumber cost with dynamic count.
//   Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("last-ditch-effort"),
        name: "Last-Ditch Effort".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Sacrifice any number of creatures. Last-Ditch Effort deals that much damage to any target.".to_string(),
        abilities: vec![
            // TODO: No Cost::SacrificeAnyNumber variant in DSL.
            //   Amount of damage = number of creatures sacrificed (dynamic variable).
            //   Cannot be expressed without SacrificeAnyNumber cost + EffectAmount::SacrificedCount.
        ],
        ..Default::default()
    }
}
