// Mother of Runes — {W}, Creature — Human Cleric 1/1.
// "{T}: Target creature you control gains protection from the color of your
// choice until end of turn."
// TODO: DSL gap — "color of your choice" requires interactive choice + dynamic
// ProtectionQuality grant. Only the creature body is defined.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mother-of-runes"),
        name: "Mother of Runes".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Cleric"]),
        oracle_text: "{T}: Target creature you control gains protection from the color of your choice until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![],
        // TODO: activated ability with color choice + protection grant
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
