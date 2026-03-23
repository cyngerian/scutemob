// Murmuring Mystic — {3}{U}, Creature — Human Wizard 1/5
// Whenever you cast an instant or sorcery spell, create a 1/1 blue Bird Illusion creature
// token with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("murmuring-mystic"),
        name: "Murmuring Mystic".to_string(),
        mana_cost: Some(ManaCost { generic: 3, blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Whenever you cast an instant or sorcery spell, create a 1/1 blue Bird Illusion creature token with flying.".to_string(),
        power: Some(1),
        toughness: Some(5),
        abilities: vec![
            // TODO: WheneverYouCastSpell lacks spell-type filter (instant/sorcery only).
            //   Overbroad trigger would create tokens on creature spells too — removed.
        ],
        ..Default::default()
    }
}
