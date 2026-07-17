// Sejiri Shelter // Sejiri Glacier — Target creature you control gains protection from the color of your ch
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sejiri-shelter"),
        name: "Sejiri Shelter // Sejiri Glacier".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Target creature you control gains protection from the color of your choice until end of turn.".to_string(),
        abilities: vec![],
        completeness: Completeness::inert("Blocked twice: (1) 'protection from the color of your choice' — ProtectionQuality has only FromColor(Color) fixed at authoring time; no FromChosenColor and no path from an on-resolution color choice to a protection grant (same blocker as Mother of Runes); (2) MDFC back face (Sejiri Glacier, Land, enters tapped) is not authored — back_face is None."),
        ..Default::default()
    }
}
