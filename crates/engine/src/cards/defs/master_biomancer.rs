// Master Biomancer — {2}{G}{U}, Creature — Elf Wizard 2/4
// Each other creature you control enters with a number of additional +1/+1 counters on it
//   equal to this creature's power and as a Mutant in addition to its other types.
//
// TODO: "each other creature enters with +1/+1 counters equal to this creature's power" —
//   ETB replacement effect with a dynamic counter count based on this creature's power.
//   ReplacementModification::EntersWith(EffectAmount::SourcePower) does not exist in DSL.
//   The ETB replacement with static count exists (EntersTapped, etc.) but not with dynamic amounts.
// TODO: "enters as a Mutant in addition to its other types" — type-granting ETB replacement
//   also not in DSL.
//   Both halves require new engine primitives. Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("master-biomancer"),
        name: "Master Biomancer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Wizard"]),
        oracle_text: "Each other creature you control enters with a number of additional +1/+1 counters on it equal to this creature's power and as a Mutant in addition to its other types.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // TODO: Static ETB replacement — other creatures you control enter with +1/+1 counters
            //   equal to this creature's power AND as a Mutant.
            //   DSL gap: no ReplacementModification::EntersWithCountersEqualToSourcePower or
            //   ReplacementModification::EntersAsAdditionalType. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
