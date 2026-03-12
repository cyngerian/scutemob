// Arixmethes, Slumbering Isle
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("arixmethes-slumbering-isle"),
        name: "Arixmethes, Slumbering Isle".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Kraken"]),
        oracle_text: "Arixmethes enters tapped with five slumber counters on it.\nAs long as Arixmethes has a slumber counter on it, it's a land. (It's not a creature.)\nWhenever you cast a spell, you may remove a slumber counter from Arixmethes.\n{T}: Add {G}{U}.".to_string(),
        abilities: vec![
            // TODO: Arixmethes enters tapped with five slumber counters on it.
            // TODO: Static — As long as Arixmethes has a slumber counter on it, it's a land. (It's not a crea
            // TODO: Triggered — Whenever you cast a spell, you may remove a slumber counter from Arixmethes.
            // TODO: Activated — {T}: Add {G}{U}.
        ],
        power: Some(12),
        toughness: Some(12),
        ..Default::default()
    }
}
