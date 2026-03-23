// Tragic Slip — {B}, Instant
// Target creature gets -1/-1 until end of turn.
// Morbid — That creature gets -13/-13 until end of turn instead if a creature died this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tragic-slip"),
        name: "Tragic Slip".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature gets -1/-1 until end of turn.\nMorbid — That creature gets -13/-13 until end of turn instead if a creature died this turn.".to_string(),
        // TODO: Morbid — "if a creature died this turn" → -13/-13, else -1/-1.
        // Requires Condition::CreatureDiedThisTurn. Partial -1/-1 is wrong game state
        // (KI-2) because it under-kills. Stripped per W6 policy.
        abilities: vec![],
        ..Default::default()
    }
}
