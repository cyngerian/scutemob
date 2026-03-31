// Mindcrank — {2}, Artifact
// Whenever an opponent loses life, that player mills that many cards.
//
// TODO: TriggerCondition::WheneverOpponentLosesLife does not exist in the DSL.
// The trigger fires for each opponent who loses life, and the mill amount equals
// the life lost ("that many cards") — requires both the trigger condition and
// EffectAmount::TriggeringAmount. Abilities left empty per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mindcrank"),
        name: "Mindcrank".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever an opponent loses life, that player mills that many cards. (Damage causes loss of life.)".to_string(),
        abilities: vec![
            // TODO: TriggerCondition::WheneverOpponentLosesLife not in DSL.
            // Also needs EffectAmount::TriggeringAmount for "that many cards".
            // W5: partial implementation would fire wrong effects — omitted.
        ],
        ..Default::default()
    }
}
