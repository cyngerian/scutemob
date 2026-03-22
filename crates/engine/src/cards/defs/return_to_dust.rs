// Return to Dust — {2}{W}{W}, Instant
// Exile target artifact or enchantment. If you cast this spell during your main phase,
// you may exile up to one other target artifact or enchantment.
// TODO: DSL gap — "if cast during your main phase" conditional second exile target is not
//   expressible. The second target is optional and main-phase-gated. Implementing just the
//   first exile would produce wrong game state (misses the second target slot entirely on
//   main phase casts). Per W5 policy, leaving abilities empty.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("return-to-dust"),
        name: "Return to Dust".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target artifact or enchantment. If you cast this spell during your main phase, you may exile up to one other target artifact or enchantment.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
