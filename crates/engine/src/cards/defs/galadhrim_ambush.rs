// Galadhrim Ambush — {3}{G}, Instant
// Create X 1/1 green Elf Warrior creature tokens, where X is the number of attacking creatures.
// Prevent all combat damage that would be dealt this turn by non-Elf creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("galadhrim-ambush"),
        name: "Galadhrim Ambush".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Create X 1/1 green Elf Warrior creature tokens, where X is the number of attacking creatures.\nPrevent all combat damage that would be dealt this turn by non-Elf creatures.".to_string(),
        abilities: vec![
            // TODO: "Create X tokens where X = number of attacking creatures" — EffectAmount
            // lacks an attacking-creature-count variant. DSL gap.
            // TODO: "Prevent all combat damage by non-Elf creatures" — damage prevention with
            // subtype-exclusion filter not in DSL. Per W5 policy, leaving empty.
        ],
        ..Default::default()
    }
}
