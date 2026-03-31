// Persist — {1}{B} Sorcery
// Return target nonlegendary creature card from your graveyard to the battlefield
// with a -1/-1 counter on it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("persist"),
        name: "Persist".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return target nonlegendary creature card from your graveyard to the battlefield with a -1/-1 counter on it.".to_string(),
        abilities: vec![
            // TODO: "target nonlegendary creature card" — TargetFilter has no non_legendary
            // field (only `legendary: bool` for the positive case). Implementing without the
            // nonlegendary restriction produces wrong game state (targets legendary creatures).
            // Per W5 policy, the ability is omitted. Needs TargetFilter::non_legendary: bool.
        ],
        ..Default::default()
    }
}
