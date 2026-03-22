// Teferi's Protection — {2}{W}, Instant
// Until your next turn, your life total can't change and you gain protection from everything.
// All permanents you control phase out.
// Exile Teferi's Protection.
//
// TODO: "your life total can't change" — needs a continuous prevention effect until next turn.
// TODO: "you gain protection from everything" — player protection not in DSL.
// TODO: "All permanents you control phase out" — Effect::PhaseOut for all controller permanents.
// TODO: "Exile Teferi's Protection" — self-exile on resolution.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teferis-protection"),
        name: "Teferi's Protection".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Until your next turn, your life total can't change and you gain protection from everything. All permanents you control phase out. (While they're phased out, they're treated as though they don't exist. They phase in before you untap during your untap step.)\nExile Teferi's Protection.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
