// Tear Asunder — {1}{G}, Instant
// Kicker {1}{B}
// Exile target artifact or enchantment. If this spell was kicked, exile target nonland permanent instead.
// CR 702.33a: Kicker — optional additional cost for enhanced effect.
// CR 702.33d: "kicked" means the player paid the kicker cost at cast time.
// TODO: DSL gap — Kicker changes the valid target set (artifact/enchantment vs nonland permanent).
//   The `targets` field on AbilityDefinition::Spell is fixed at cast time; there is no way to
//   express "target requirement depends on whether kicker was paid." Neither filter is correct
//   for both modes. Per W5 policy, leaving abilities empty rather than producing wrong game state.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tear-asunder"),
        name: "Tear Asunder".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Kicker {1}{B} (You may pay an additional {1}{B} as you cast this spell.)\nExile target artifact or enchantment. If this spell was kicked, exile target nonland permanent instead.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
